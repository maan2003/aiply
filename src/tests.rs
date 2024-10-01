use crate::instruction_parser::parse_instruction_symbols;
use crate::markdown_parser::ParsedLlmOutput;
use crate::{CodeParsingContext, Symbol};
use clap::Parser;
use std::fs;

fn extract_symbols(
    parsing_ctx: &mut CodeParsingContext,
    parsed_output: &ParsedLlmOutput,
) -> RelevantSymbols {
    let mut instruction_symbols = Vec::new();
    let mut code_symbols = Vec::new();

    // Extract symbols from instructions
    for instruction in &parsed_output.instructions {
        instruction_symbols.extend(parse_instruction_symbols(&instruction.text));
    }

    // Extract symbols from code changes
    for code_change in &parsed_output.code_changes {
        code_symbols
            .extend(parsing_ctx.parse_code_symbols(&code_change.language, &code_change.code));
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
struct RelevantSymbols {
    instruction_symbols: Vec<Symbol>,
    code_symbols: Vec<Symbol>,
}

#[test]
fn test_parse_llm_output() {
    let test_cases = fs::read_dir("src/tests/inputs")
        .expect("Failed to read test inputs directory")
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().to_str().map(String::from));
    let mut ctx = CodeParsingContext::new();

    for case in test_cases {
        let input = fs::read_to_string(format!("src/tests/inputs/{}", case))
            .expect("Failed to read test input file");
        let parsed_output = ParsedLlmOutput::parse(&input);

        insta::with_settings!({
            snapshot_path => "tests/snapshots",
            prepend_module_to_snapshot => false,
        }, {
            insta::assert_debug_snapshot!(&*case, (&parsed_output, extract_symbols(&mut ctx, &parsed_output)));
        });
    }
}
