pub mod instruction_parser;
pub mod markdown_parser;

use tree_sitter::{Parser as TsParser, Query, QueryCursor};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Range {
    start: usize,
    end: usize,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol {
    pub parts: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SymbolWithRange {
    symbol: Symbol,
    range: Range,
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.parts.join("::"))
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

        let mut symbols_with_range = Vec::new();
        let mut query_cursor = QueryCursor::new();
        for m in query_cursor.matches(&self.query, root_node, code.as_bytes()) {
            let mut name = None;
            let mut range = None;

            for capture in m.captures {
                match self.query.capture_names()[capture.index as usize].as_str() {
                    "name" => name = Some((&code[capture.node.byte_range()]).to_string()),
                    "item" => {
                        let byte_range = capture.node.byte_range();
                        range = Some(Range {
                            start: byte_range.start,
                            end: byte_range.end,
                        });
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (name, range) {
                symbols_with_range.push(SymbolWithRange {
                    symbol: Symbol { parts: vec![name] },
                    range,
                });
            }
        }

        self.process_symbols(symbols_with_range)
    }

    fn process_symbols(&self, mut symbols_with_range: Vec<SymbolWithRange>) -> Vec<Symbol> {
        symbols_with_range.sort_by_key(|s| (s.range.start, std::cmp::Reverse(s.range.end)));

        let mut stack = Vec::<SymbolWithRange>::new();
        let mut result = Vec::new();

        for mut symbol_with_range in symbols_with_range {
            while let Some(last) = stack.last() {
                // not overlapping
                if last.range.end < symbol_with_range.range.end {
                    stack.pop();
                } else {
                    break;
                }
            }

            if let Some(parent) = stack.last() {
                symbol_with_range.symbol.parts = parent
                    .symbol
                    .parts
                    .iter()
                    .chain(symbol_with_range.symbol.parts.iter())
                    .cloned()
                    .collect();
            }

            result.push(symbol_with_range.symbol.clone());
            stack.push(symbol_with_range.clone());
        }

        result
    }
}

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_code_symbols_empty() {
        let mut context = CodeParsingContext::new();
        let symbols = context.parse_code_symbols("rust", "");
        assert_eq!(symbols.len(), 0);
    }
}
