#![allow(dead_code)]
use clap::{Parser, Subcommand};
use pulldown_cmark::{CodeBlockKind, Event, Parser as MarkdownParser, Tag, TagEnd};
use regex::Regex;
use std::path::PathBuf;
use std::sync::OnceLock;
use tree_sitter::{Parser as TsParser, Query, QueryCursor};

struct ParsingContext {
    parser: TsParser,
    query: Query,
}

impl ParsingContext {
    fn new() -> Self {
        let mut parser = TsParser::new();
        parser
            .set_language(tree_sitter_rust::language())
            .expect("Error loading Rust grammar");

        let query_source = include_str!("rust_query.scm");
        let query = Query::new(tree_sitter_rust::language(), &query_source)
            .expect("Failed to create query");

        ParsingContext { parser, query }
    }

    fn parse_code_symbols(&mut self, code: &str) -> Vec<Symbol> {
        let tree = self.parser.parse(code, None).unwrap();
        let root_node = tree.root_node();

        let mut symbols = Vec::new();
        let mut query_cursor = QueryCursor::new();
        for m in query_cursor.matches(&self.query, root_node, code.as_bytes()) {
            for capture in m.captures {
                let name = &code[capture.node.byte_range()];
                match capture.index {
                    0 | 1 | 2 | 3 | 4 | 5 => symbols.push(Symbol {
                        container: None,
                        name: name.to_string(),
                    }),
                    6 => {
                        let container = m
                            .captures
                            .iter()
                            .find(|c| c.index == 7)
                            .map(|c| &code[c.node.byte_range()]);
                        symbols.push(Symbol {
                            container: container.map(|c| c.to_string()),
                            name: name.to_string(),
                        });
                    }
                    8 => {
                        let container = m
                            .captures
                            .iter()
                            .find(|c| c.index == 9)
                            .map(|c| &code[c.node.byte_range()]);
                        symbols.push(Symbol {
                            container: container.map(|c| c.to_string()),
                            name: name.to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }

        symbols
    }

    fn parse_instruction_symbols(&self, text: &str) -> Vec<Symbol> {
        static SYMBOL_REGEX: OnceLock<Regex> = OnceLock::new();
        let symbol_pattern = SYMBOL_REGEX.get_or_init(|| {
            Regex::new(
                r#"(?x)
                \b(?:
                    [a-z0-9]+(?:(?:::[a-z0-9_A-Z]*|_[a-z0-9]+))+
                   |
                    [A-Z][a-z0-9]*(?:(?:::[a-z0-9_A-Z]*|[A-Z][a-z0-9]*))+
                  )
                \b
            "#,
            )
            .unwrap()
        });

        symbol_pattern
            .find_iter(text)
            .map(|m| {
                let s = m.as_str();
                if let Some((container, name)) = s.rsplit_once("::") {
                    Symbol {
                        container: Some(container.to_string()),
                        name: name.to_string(),
                    }
                } else {
                    Symbol {
                        container: None,
                        name: s.to_string(),
                    }
                }
            })
            .collect()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Symbol {
    container: Option<String>,
    name: String,
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.container {
            Some(container) => write!(f, "#{}::{}", container, self.name),
            None => write!(f, "#{}", self.name),
        }
    }
}

fn extract_symbols(
    parsing_ctx: &mut ParsingContext,
    parsed_output: &ParsedOutput,
) -> RelevantSymbols {
    let mut instruction_symbols = Vec::new();
    let mut code_symbols = Vec::new();

    // Extract symbols from instructions
    for instruction in &parsed_output.instructions {
        instruction_symbols.extend(parsing_ctx.parse_instruction_symbols(&instruction.text));
    }

    // Extract symbols from code changes
    for code_change in &parsed_output.code_changes {
        if code_change.language.to_lowercase() == "rust" {
            code_symbols.extend(parsing_ctx.parse_code_symbols(&code_change.code));
        }
    }

    instruction_symbols.sort();
    instruction_symbols.dedup();
    code_symbols.sort();
    code_symbols.dedup();

    RelevantSymbols {
        instruction_symbols,
        code_symbols,
    }
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
}

#[derive(Clone, Debug)]
struct RelevantSymbols {
    instruction_symbols: Vec<Symbol>,
    code_symbols: Vec<Symbol>,
}

fn parse_llm_output(output: &str) -> ParsedOutput {
    let parser = MarkdownParser::new(output);
    let mut parsed_output = ParsedOutput {
        instructions: Vec::new(),
        code_changes: Vec::new(),
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
                }
            }
            Event::Code(code) => {
                if !in_code_block {
                    current_instruction.push_str(&format!("`{}`", code));
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
    let mut parsing_ctx = ParsingContext::new();

    match &cli.command {
        Commands::Parse { file } => {
            let content = std::fs::read_to_string(file)?;
            let parsed_output = parse_llm_output(&content);
            let relevant_symbols = extract_symbols(&mut parsing_ctx, &parsed_output);
            println!("{relevant_symbols:#?}");
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
        let mut ctx = ParsingContext::new();

        for case in test_cases {
            let input = fs::read_to_string(format!("src/tests/inputs/{}", case))
                .expect("Failed to read test input file");
            let parsed_output = parse_llm_output(&input);

            insta::with_settings!({
                snapshot_path => "tests/snapshots",
                prepend_module_to_snapshot => false,
            }, {
                insta::assert_debug_snapshot!(&*case, (&parsed_output, extract_symbols(&mut ctx, &parsed_output)));
            });
        }
    }

    #[test]
    fn test_parse_instruction_symbols() {
        let ctx = ParsingContext::new();
        let test_cases = vec![
            ("HelloWorld FooBar", vec!["#HelloWorld", "#FooBar"]),
            ("hello_world foo_bar", vec!["#hello_world", "#foo_bar"]),
            (
                "Foo::Bar Baz::Qux::Quux",
                vec!["#Foo::Bar", "#Baz::Qux::Quux"],
            ),
            ("BTreeMap::raw_insert", vec!["#BTreeMap::raw_insert"]),
            ("BTreeMap", vec!["#BTreeMap"]),
            (
                "HelloWorld snake_case Foo::Bar",
                vec!["#HelloWorld", "#snake_case", "#Foo::Bar"],
            ),
            ("hello world", vec![]),
            ("Hello World", vec![]),
            (
                "Symbols with numbers: Hello123World snake_case_42",
                vec!["#Hello123World", "#snake_case_42"],
            ),
        ];

        for (input, expected) in test_cases {
            let result = ctx
                .parse_instruction_symbols(input)
                .into_iter()
                .map(|s| format!("{s:?}"))
                .collect::<Vec<_>>();
            assert_eq!(result, expected, "Failed on input: {}", input);
        }
    }
}
