use pulldown_cmark::{CodeBlockKind, Event, Parser as MarkdownParser, Tag, TagEnd};

#[derive(Clone, Debug)]
pub struct Instruction {
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct CodeChange {
    pub language: String,
    pub code: String,
}

#[derive(Clone, Debug)]
pub struct ParsedLlmOutput {
    pub instructions: Vec<Instruction>,
    pub code_changes: Vec<CodeChange>,
}

impl ParsedLlmOutput {
    pub fn parse(output: &str) -> ParsedLlmOutput {
        let parser = MarkdownParser::new(output);
        let mut parsed_output = ParsedLlmOutput {
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
}
