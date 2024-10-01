pub mod instruction_parser;
pub mod markdown_parser;

use tree_sitter::{Parser as TsParser, Query, QueryCursor};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol {
    pub container: Option<String>,
    pub name: String,
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.container {
            Some(container) => write!(f, "#{}::{}", container, self.name),
            None => write!(f, "#{}", self.name),
        }
    }
}

pub struct CodeParsingContext {
    parser: TsParser,
    query: Query,
}

impl CodeParsingContext {
    pub fn new() -> Self {
        let mut parser = TsParser::new();
        parser
            .set_language(tree_sitter_rust::language())
            .expect("Error loading Rust grammar");

        let query_source = include_str!("rust_query.scm");
        let query = Query::new(tree_sitter_rust::language(), &query_source)
            .expect("Failed to create query");

        CodeParsingContext { parser, query }
    }

    pub fn parse_code_symbols(&mut self, language: &str, code: &str) -> Vec<Symbol> {
        if language != "rust" {
            // TODO: support for other languages
            return vec![];
        }
        let tree = self.parser.parse(code, None).unwrap();
        let root_node = tree.root_node();

        let mut symbols = Vec::new();
        let mut query_cursor = QueryCursor::new();
        for m in query_cursor.matches(&self.query, root_node, code.as_bytes()) {
            let mut container = None;
            let mut name = None;
            let mut is_item = false;

            for capture in m.captures {
                let capture_text = &code[capture.node.byte_range()];
                match self.query.capture_names()[capture.index as usize].as_str() {
                    "name" => name = Some(capture_text.to_string()),
                    "item" => is_item = true,
                    _ => {}
                }
            }

            if is_item && name.is_some() {
                symbols.push(Symbol {
                    container,
                    name: name.unwrap(),
                });
            }
        }

        symbols
    }
}

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_code_symbols() {
        let mut context = CodeParsingContext::new();

        let code = r#"
            struct MyStruct {
                field: i32,
            }

            fn my_function() {
                println!("Hello, world!");
            }

            enum MyEnum {
                VariantA,
                VariantB,
            }
        "#;

        let symbols = context.parse_code_symbols("rust", code);

        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "MyStruct");
        assert_eq!(symbols[1].name, "my_function");
        assert_eq!(symbols[2].name, "MyEnum");
    }

    #[test]
    fn test_parse_code_symbols_empty() {
        let mut context = CodeParsingContext::new();
        let symbols = context.parse_code_symbols("rust", "");
        assert_eq!(symbols.len(), 0);
    }

    #[test]
    fn test_parse_code_symbols_unsupported_language() {
        let mut context = CodeParsingContext::new();
        let symbols = context.parse_code_symbols("python", "def my_function():\n    pass");
        assert_eq!(symbols.len(), 0);
    }
}
