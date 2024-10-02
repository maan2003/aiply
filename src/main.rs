use aiply::instruction_parser::parse_instruction_symbols;
use aiply::markdown_parser::ParsedLlmOutput;
use aiply::{llm, CodeParsingContext};
use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the LLM output file
    #[arg(short, long)]
    llm_output: PathBuf,

    /// Path to the original source code file
    #[arg(short, long)]
    source_file: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let llm_output = fs::read_to_string(&cli.llm_output)
        .with_context(|| format!("Failed to read LLM output file: {:?}", cli.llm_output))?;

    let source_code = fs::read_to_string(&cli.source_file)
        .with_context(|| format!("Failed to read source code file: {:?}", cli.source_file))?;

    let parsed_output = ParsedLlmOutput::parse(&llm_output);
    let mut context = CodeParsingContext::new();

    let mut important_symbols = vec![];
    for code_changes in parsed_output.code_changes {
        important_symbols
            .extend(context.parse_code_symbols(&code_changes.language, &code_changes.code))
    }
    for instructions in parsed_output.instructions {
        important_symbols.extend(parse_instruction_symbols(&instructions.text));
    }

    let start = std::time::Instant::now();
    let collapsed_doc = context.collapse_unrelated_symbols(&source_code, important_symbols);
    let duration = start.elapsed();
    eprintln!("Time taken to collapse unrelated symbols: {:?}", duration);
    let collapsed_text = collapsed_doc.collapsed_document();
    let start = std::time::Instant::now();
    let response = llm::prompt_for_edits(&collapsed_text, &llm_output)?;
    let duration = start.elapsed();
    eprintln!("Time taken to prompt for edits: {:?}", duration);
    let start = std::time::Instant::now();
    let uncollapsed = collapsed_doc.uncollapse_document(&response);
    let duration = start.elapsed();
    eprintln!("Time taken to uncollapse document: {:?}", duration);
    println!("{uncollapsed}");

    Ok(())
}
