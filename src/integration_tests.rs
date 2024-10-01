use crate::instruction_parser::parse_instruction_symbols;
use crate::markdown_parser::ParsedLlmOutput;
use crate::{CodeParsingContext, Symbol};
use std::fs;

fn run_test(input: &str) -> TestOutput {
    let mut ctx = CodeParsingContext::new();
    let llm_output = ParsedLlmOutput::parse(input);
    let mut instruction_symbols = Vec::new();
    let mut code_symbols = Vec::new();

    // Extract symbols from instructions
    for instruction in &llm_output.instructions {
        instruction_symbols.extend(parse_instruction_symbols(&instruction.text));
    }

    // Extract symbols from code changes
    for code_change in &llm_output.code_changes {
        code_symbols.extend(ctx.parse_code_symbols(&code_change.language, &code_change.code));
    }

    instruction_symbols.sort();
    instruction_symbols.dedup();
    code_symbols.sort();
    code_symbols.dedup();

    TestOutput {
        llm_output,
        instruction_symbols,
        code_symbols,
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct TestOutput {
    llm_output: ParsedLlmOutput,
    instruction_symbols: Vec<Symbol>,
    code_symbols: Vec<Symbol>,
}

#[test]
fn test_parse_llm_output() {
    let test_cases = fs::read_dir("src/tests/inputs")
        .expect("Failed to read test inputs directory")
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().to_str().map(String::from));

    for case in test_cases {
        let input = fs::read_to_string(format!("src/tests/inputs/{}", case))
            .expect("Failed to read test input file");

        insta::with_settings!({
            snapshot_path => "tests/snapshots",
            prepend_module_to_snapshot => false,
        }, {
            insta::assert_debug_snapshot!(&*case, run_test(&input));
        });
    }
}
