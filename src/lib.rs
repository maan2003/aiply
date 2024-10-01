pub mod instruction_parser;
pub mod markdown_parser;

use std::ops::Range;

use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol {
    pub parts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SymbolWithRange {
    symbol: Symbol,
    range: Range<usize>,
    summary_range: Range<usize>,
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.parts.join("::"))
    }
}

pub struct CodeParsingContext {
    parser: Parser,
    query: Query,
}

pub struct CollapsedDocument<'a> {
    original_document: &'a str,
    // invariant: non overlapping, sorted
    collapsed_sections: Vec<Range<usize>>,
}

impl<'a> CollapsedDocument<'a> {
    pub fn collapsed_document(&self) -> String {
        let mut result = String::new();
        let mut last_end = 0;

        for section in &self.collapsed_sections {
            // Add uncollapsed content
            result.push_str(&self.original_document[last_end..section.start]);
            // Add ellipsis for collapsed content
            result.push_str("...");
            last_end = section.end;
        }

        // Add remaining uncollapsed content
        result.push_str(&self.original_document[last_end..]);

        result
    }
}

impl CodeParsingContext {
    pub fn new() -> Self {
        let mut parser = Parser::new();
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
        let symbols_with_range = self.extract_symbols_with_range(code);
        self.process_symbols(symbols_with_range)
            .into_iter()
            .map(|x| x.symbol)
            .collect()
    }

    fn extract_symbols_with_range(&mut self, code: &str) -> Vec<SymbolWithRange> {
        let tree = self.parser.parse(code, None).unwrap();
        let root_node = tree.root_node();
        let mut symbols_with_range = Vec::new();
        let mut query_cursor = QueryCursor::new();
        for m in query_cursor.matches(&self.query, root_node, code.as_bytes()) {
            let mut name = None;
            let mut summary_start = usize::MAX;
            let mut summary_end = 0;
            let mut range_start = usize::MAX;
            let mut range_end = 0;

            for capture in m.captures {
                let byte_range = capture.node.byte_range();
                let capture_name = self.query.capture_names()[capture.index as usize].as_str();
                dbg!(capture_name, capture.node.to_sexp());
                match capture_name {
                    "name" => {
                        name = Some((&code[byte_range.clone()]).to_string());
                        summary_start = summary_start.min(byte_range.start);
                        summary_end = summary_end.max(byte_range.end);
                    }
                    "context" => {
                        summary_start = summary_start.min(byte_range.start);
                        summary_end = summary_end.max(byte_range.end);
                    }
                    "item" => {
                        range_start = range_start.min(byte_range.start);
                        range_end = range_end.max(byte_range.end);
                    }
                    _ => {}
                }
            }

            let item_range = Range {
                start: range_start,
                end: range_end,
            };

            if let Some(name) = name {
                symbols_with_range.push(SymbolWithRange {
                    symbol: Symbol { parts: vec![name] },
                    range: item_range,
                    summary_range: Range {
                        start: summary_start,
                        end: summary_end,
                    },
                });
            }
        }

        symbols_with_range
    }

    pub fn collapse_unrelated_symbols<'a>(
        &mut self,
        original_doc: &'a str,
        important_symbols: Vec<Symbol>,
    ) -> CollapsedDocument<'a> {
        let symbols_with_range = self.extract_symbols_with_range(original_doc);
        let processed_symbols = self.process_symbols(symbols_with_range);
        let mut collapsed_sections = Vec::new();

        for symbol in processed_symbols {
            if !important_symbols
                .iter()
                .any(|important| self.symbols_match(&symbol.symbol, important))
            {
                // Collapse the range that's not part of the summary
                if symbol.range.start < symbol.summary_range.start {
                    collapsed_sections.push(Range {
                        start: symbol.range.start,
                        end: symbol.summary_range.start,
                    });
                }
                if symbol.summary_range.end < symbol.range.end {
                    collapsed_sections.push(Range {
                        start: symbol.summary_range.end,
                        end: symbol.range.end,
                    });
                }
            }
        }

        // Merge overlapping or adjacent ranges
        collapsed_sections.sort_by_key(|r| r.start);
        let mut merged_sections: Vec<Range<usize>> = Vec::new();
        for section in collapsed_sections {
            if let Some(last) = merged_sections.last_mut() {
                if last.end >= section.start {
                    last.end = last.end.max(section.end);
                } else {
                    merged_sections.push(section);
                }
            } else {
                merged_sections.push(section);
            }
        }

        CollapsedDocument {
            original_document: original_doc,
            collapsed_sections: merged_sections,
        }
    }

    fn symbols_match(&self, symbol: &Symbol, important: &Symbol) -> bool {
        if symbol.parts.len() > important.parts.len() {
            return false;
        }

        for (s, i) in symbol.parts.iter().zip(important.parts.iter()) {
            if s != i {
                return false;
            }
        }
        true
    }

    fn process_symbols(
        &self,
        mut symbols_with_range: Vec<SymbolWithRange>,
    ) -> Vec<SymbolWithRange> {
        symbols_with_range.sort_by_key(|s| (s.range.start, std::cmp::Reverse(s.range.end)));

        let mut stack = Vec::<SymbolWithRange>::new();
        let mut result = Vec::new();

        for mut symbol_with_range in symbols_with_range {
            while let Some(last) = stack.last() {
                if last.range.end <= symbol_with_range.range.start {
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

            result.push(symbol_with_range.clone());
            stack.push(symbol_with_range);
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
