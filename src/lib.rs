pub mod instruction_parser;
pub mod llm;
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
    collapse_query: Query,
}

#[derive(Clone)]
pub enum CollapseReplacement {
    Range(Range<usize>),
    Imports,
}

#[derive(Clone)]
pub struct Collapse {
    replacement: CollapseReplacement,
    target: Range<usize>,
}

pub struct CollapsedDocument<'a> {
    original_document: &'a str,
    // invariant: non overlapping, sorted
    collapses: Vec<Collapse>,
}

impl<'a> CollapsedDocument<'a> {
    pub fn collapsed_document(&self) -> String {
        let mut result = String::new();
        let mut last_end = 0;

        for collapse in &self.collapses {
            // Add uncollapsed content
            result.push_str(&self.original_document[last_end..collapse.target.start]);
            // Add replacement content
            match &collapse.replacement {
                CollapseReplacement::Range(range) => {
                    result.push_str(&self.original_document[range.clone()]);
                    result.push_str(" ...");
                }
                CollapseReplacement::Imports => {
                    result.push_str("use ...");
                }
            }
            last_end = collapse.target.end;
        }

        // Add remaining uncollapsed content
        result.push_str(&self.original_document[last_end..]);

        result
    }

    pub fn uncollapse_document(&self, new_collapsed: &str) -> String {
        let mut result = String::new();
        let mut collapses = self.collapses.clone();
        for line in new_collapsed.lines() {
            if line.ends_with("...") {
                let prefix = line.trim_end_matches("...");
                if let Some((index, collapse)) =
                    collapses
                        .iter()
                        .enumerate()
                        .find(|(_, c)| match &c.replacement {
                            CollapseReplacement::Range(range) => {
                                dbg!(&self.original_document[range.clone()]) == prefix.trim()
                            }
                            CollapseReplacement::Imports => prefix.trim() == "use",
                        })
                {
                    let indent = prefix.len() - prefix.trim_start().len();
                    result.push_str(&prefix[..indent]);
                    // Use the target range for uncollapsing
                    result.push_str(&self.original_document[collapse.target.clone()]);
                    // Remove the matched collapse to avoid duplicate matches
                    collapses.remove(index);
                } else {
                    panic!("not found `{prefix}`");
                    // If no matching collapse is found, keep the original line
                    result.push_str(line);
                }
            } else {
                // For lines without "...", keep them as is
                result.push_str(line);
            }
            result.push('\n');
        }
        result
    }
}

impl CodeParsingContext {
    pub fn new(language: &str) -> Self {
        let mut parser = Parser::new();
        let ts_language = match language {
            "rust" => tree_sitter_rust::LANGUAGE,
            "typescript" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            _ => panic!("Unsupported language"),
        };
        let ts_language = tree_sitter::Language::new(ts_language);
        parser
            .set_language(&ts_language)
            .expect("Error loading language grammar");

        let query_source = match language {
            "rust" => include_str!("rust_query.scm"),
            "typescript" => include_str!("ts_query.scm"),
            _ => panic!("Unsupported language"),
        };
        let query = Query::new(&ts_language, query_source).expect("Failed to create query");
        let collapse_query = Query::new(
            &ts_language,
            match language {
                "rust" => "(use_declaration)+ @collapse",
                "typescript" => "(import_statement)+ @collapse",
                _ => panic!("Unsupported language"),
            },
        )
        .expect("Failed to create query");

        CodeParsingContext {
            parser,
            query,
            collapse_query,
        }
    }

    pub fn parse_code_symbols(&mut self, code: &str) -> Vec<Symbol> {
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
                let capture_name = self.query.capture_names()[capture.index as usize];
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
        let mut collapses = Vec::new();
        let tree = self.parser.parse(original_doc, None).unwrap();
        let root_node = tree.root_node();
        let mut query_cursor = QueryCursor::new();
        for m in query_cursor.matches(&self.collapse_query, root_node, original_doc.as_bytes()) {
            let mut start = usize::MAX;
            let mut end = 0;
            for capture in m.captures {
                let byte_range = capture.node.byte_range();
                start = start.min(byte_range.start);
                end = end.max(byte_range.end);
            }
            if start < end {
                collapses.push(Collapse {
                    replacement: CollapseReplacement::Imports,
                    target: start..end,
                });
            }
        }

        for symbol in processed_symbols {
            if !important_symbols
                .iter()
                .any(|important| self.symbols_match(&symbol.symbol, important))
            {
                // Collapse the range that's not part of the summary
                if symbol.range.start < symbol.summary_range.start
                    || symbol.range.end > symbol.summary_range.end
                {
                    collapses.push(Collapse {
                        replacement: CollapseReplacement::Range(
                            symbol.summary_range.start..symbol.summary_range.end,
                        ),
                        target: symbol.range.start..symbol.range.end,
                    });
                }
            }
        }

        // Merge overlapping or adjacent collapses
        collapses.sort_by_key(|c| c.target.start);
        let mut merged_collapses: Vec<Collapse> = Vec::new();
        for collapse in collapses {
            if let Some(last) = merged_collapses.last_mut() {
                if last.target.end >= collapse.target.start {
                    last.target.end = last.target.end.max(collapse.target.end);
                } else {
                    merged_collapses.push(collapse);
                }
            } else {
                merged_collapses.push(collapse);
            }
        }

        CollapsedDocument {
            original_document: original_doc,
            collapses: merged_collapses,
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
        let mut context = CodeParsingContext::new("rust");
        let symbols = context.parse_code_symbols("");
        assert_eq!(symbols.len(), 0);
    }
}
