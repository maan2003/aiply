use aiply::instruction_parser::parse_instruction_symbols;
use aiply::markdown_parser::ParsedLlmOutput;
use aiply::{llm, CodeParsingContext};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the full LLM-based code editing process
    Edit(EditArgs),
    /// Only print the collapsed document
    Collapse(CollapseArgs),
}

#[derive(Parser)]
struct EditArgs {
    /// Path to the LLM output file
    #[arg(short, long)]
    llm_output: PathBuf,

    /// Path to the original source code file
    #[arg(short, long)]
    source_file: PathBuf,

    #[arg(short, long)]
    language: String,
}

#[derive(Parser)]
struct CollapseArgs {
    /// Path to the LLM output file
    #[arg(short, long)]
    llm_output: PathBuf,

    /// Path to the original source code file
    #[arg(short, long)]
    source_file: PathBuf,

    #[arg(short, long)]
    language: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Edit(args) => run_edit(args),
        Commands::Collapse(args) => run_collapse(args),
    }
}

fn run_edit(args: EditArgs) -> Result<()> {
    let llm_output = fs::read_to_string(&args.llm_output)
        .with_context(|| format!("Failed to read LLM output file: {:?}", args.llm_output))?;

    let source_code = fs::read_to_string(&args.source_file)
        .with_context(|| format!("Failed to read source code file: {:?}", args.source_file))?;

    let parsed_output = ParsedLlmOutput::parse(&llm_output);
    let mut context = CodeParsingContext::new(&args.language);

    let mut important_symbols = vec![];
    for code_changes in parsed_output.code_changes {
        important_symbols.extend(context.parse_code_symbols(&code_changes.code))
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
    let response = llm::prompt_for_edits(&args.language, &collapsed_text, &llm_output)?;
    let duration = start.elapsed();
    eprintln!("Time taken to prompt for edits: {:?}", duration);
    let start = std::time::Instant::now();
    let uncollapsed = collapsed_doc.uncollapse_document(&response);
    let duration = start.elapsed();
    eprintln!("Time taken to uncollapse document: {:?}", duration);
    println!("{uncollapsed}");

    Ok(())
}

fn run_collapse(args: CollapseArgs) -> Result<()> {
    let llm_output = fs::read_to_string(&args.llm_output)
        .with_context(|| format!("Failed to read LLM output file: {:?}", args.llm_output))?;

    let source_code = fs::read_to_string(&args.source_file)
        .with_context(|| format!("Failed to read source code file: {:?}", args.source_file))?;

    let parsed_output = ParsedLlmOutput::parse(&llm_output);
    let mut context = CodeParsingContext::new(&args.language);

    let mut important_symbols = vec![];
    for code_changes in parsed_output.code_changes {
        important_symbols.extend(context.parse_code_symbols(&code_changes.code))
    }
    for instructions in parsed_output.instructions {
        important_symbols.extend(parse_instruction_symbols(&instructions.text));
    }

    let start = std::time::Instant::now();
    let collapsed_doc = context.collapse_unrelated_symbols(&source_code, important_symbols);
    let duration = start.elapsed();
    eprintln!("Time taken to collapse unrelated symbols: {:?}", duration);
    let collapsed_text = collapsed_doc.collapsed_document();
    println!("{collapsed_text}");

    Ok(())
}
