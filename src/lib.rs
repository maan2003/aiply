use tree_sitter::{Parser as TsParser, Query, QueryCursor};

mod instruction_parser;
mod markdown_parser;
#[cfg(test)]
mod tests;

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
            // TODO: support for other langauges
            return vec![];
        }
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
}
