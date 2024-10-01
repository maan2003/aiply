#![allow(dead_code)]
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use pulldown_cmark::{CodeBlockKind, Event, Parser as MarkdownParser, Tag, TagEnd};
use regex::Regex;

fn parse_code_symbols(text: &str) -> Vec<String> {
    let symbol_pattern = Regex::new(
        r#"(?x)
        \b(?:
            [a-z0-9]+(?:(?:::[a-z0-9_A-Z]*|_[a-z0-9]+))+
           |
            [A-Z][a-z0-9]*(?:(?:::[a-z0-9_A-Z]*|[A-Z][a-z0-9]*))+
          )
        \b
    "#,
    )
    .unwrap();

    symbol_pattern
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

#[derive(Clone, Debug)]
struct Instruction {
    text: String,
}

#[derive(Clone, Debug)]
struct CodeChange {
    language: String,
    code: String,
}

#[derive(Clone, Debug)]
struct ParsedOutput {
    instructions: Vec<Instruction>,
    code_changes: Vec<CodeChange>,
    code_symbols: Vec<String>,
}

fn parse_llm_output(output: &str) -> ParsedOutput {
    let parser = MarkdownParser::new(output);
    let mut parsed_output = ParsedOutput {
        instructions: Vec::new(),
        code_changes: Vec::new(),
        code_symbols: Vec::new(),
    };
    let mut current_instruction = String::new();
    let mut in_code_block = false;
    let mut current_code_change = CodeChange {
        language: String::new(),
        code: String::new(),
    };

    for event in parser {
        match event {
            Event::Text(text) => {
                if in_code_block {
                    current_code_change.code.push_str(&text);
                } else {
                    current_instruction.push_str(&text);
                    parsed_output.code_symbols.extend(parse_code_symbols(&text));
                }
            }
            Event::Code(code) => {
                if !in_code_block {
                    current_instruction.push_str(&format!("`{}`", code));
                    parsed_output.code_symbols.push(code.to_string());
                }
            }
            Event::Start(Tag::CodeBlock(lang)) => {
                in_code_block = true;
                current_code_change.language = match lang {
                    CodeBlockKind::Indented => String::new(),
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                parsed_output.code_changes.push(current_code_change.clone());
                current_code_change = CodeChange {
                    language: String::new(),
                    code: String::new(),
                };
                if !current_instruction.is_empty() {
                    parsed_output.instructions.push(Instruction {
                        text: current_instruction.trim().to_string(),
                    });
                    current_instruction.clear();
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if !in_code_block {
                    current_instruction.push('\n');
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if !current_instruction.is_empty() {
                    parsed_output.instructions.push(Instruction {
                        text: current_instruction.trim().to_string(),
                    });
                    current_instruction.clear();
                }
            }
            _ => {}
        }
    }

    if !current_instruction.is_empty() {
        parsed_output.instructions.push(Instruction {
            text: current_instruction.trim().to_string(),
        });
    }

    parsed_output.code_symbols.sort();
    parsed_output.code_symbols.dedup();

    parsed_output
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Parse { file: PathBuf },
}

fn main() -> std::io::Result<()> {
    let cli: Cli = Cli::parse();

    match &cli.command {
        Commands::Parse { file } => {
            let content = std::fs::read_to_string(file)?;
            let parsed_output = parse_llm_output(&content);
            println!("{parsed_output:#?}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    #[test]
    fn test_parse_llm_output() {
        let test_cases = fs::read_dir("src/tests/inputs")
            .expect("Failed to read test inputs directory")
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().to_str().map(String::from));

        for case in test_cases {
            let input = fs::read_to_string(format!("src/tests/inputs/{}", case))
                .expect("Failed to read test input file");
            let parsed_output = parse_llm_output(&input);

            insta::with_settings!({
                snapshot_path => "tests/snapshots",
                prepend_module_to_snapshot => false,
            }, {
                insta::assert_debug_snapshot!(case, parsed_output);
            });
        }
    }

    #[test]
    fn test_parse_code_symbols() {
        let test_cases = vec![
            ("HelloWorld FooBar", vec!["HelloWorld", "FooBar"]),
            ("hello_world foo_bar", vec!["hello_world", "foo_bar"]),
            (
                "Foo::Bar Baz::Qux::Quux",
                vec!["Foo::Bar", "Baz::Qux::Quux"],
            ),
            ("BTreeMap::raw_insert", vec!["BTreeMap::raw_insert"]),
            ("BTreeMap", vec!["BTreeMap"]),
            (
                "HelloWorld snake_case Foo::Bar",
                vec!["HelloWorld", "snake_case", "Foo::Bar"],
            ),
            ("hello world", vec![]),
            ("Hello World", vec![]),
            (
                "Symbols with numbers: Hello123World snake_case_42",
                vec!["Hello123World", "snake_case_42"],
            ),
        ];

        for (input, expected) in test_cases {
            let result = parse_code_symbols(input);
            assert_eq!(result, expected, "Failed on input: {}", input);
        }
    }
}
