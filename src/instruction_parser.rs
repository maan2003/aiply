use crate::Symbol;
use regex::Regex;
use std::sync::OnceLock;

pub fn parse_instruction_symbols(text: &str) -> Vec<Symbol> {
    static SYMBOL_REGEX: OnceLock<Regex> = OnceLock::new();
    let symbol_pattern = SYMBOL_REGEX.get_or_init(|| {
        Regex::new(
            r#"(?x)
            \b(?:
                [a-z0-9]+(?:(?:::[a-z0-9_A-Z]*|_[a-z0-9]+))+
               |
                [A-Z][a-z0-9]*(?:(?:::[a-z0-9_A-Z]*|[A-Z][a-z0-9]*))+
              )
            \b
        "#,
        )
        .unwrap()
    });

    symbol_pattern
        .find_iter(text)
        .map(|m| {
            let s = m.as_str();
            if let Some((container, name)) = s.rsplit_once("::") {
                Symbol {
                    container: Some(container.to_string()),
                    name: name.to_string(),
                }
            } else {
                Symbol {
                    container: None,
                    name: s.to_string(),
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_instruction_symbols() {
        let test_cases = vec![
            ("HelloWorld FooBar", vec!["#HelloWorld", "#FooBar"]),
            ("hello_world foo_bar", vec!["#hello_world", "#foo_bar"]),
            (
                "Foo::Bar Baz::Qux::Quux",
                vec!["#Foo::Bar", "#Baz::Qux::Quux"],
            ),
            ("BTreeMap::raw_insert", vec!["#BTreeMap::raw_insert"]),
            ("BTreeMap", vec!["#BTreeMap"]),
            (
                "HelloWorld snake_case Foo::Bar",
                vec!["#HelloWorld", "#snake_case", "#Foo::Bar"],
            ),
            ("hello world", vec![]),
            ("Hello World", vec![]),
            (
                "Symbols with numbers: Hello123World snake_case_42",
                vec!["#Hello123World", "#snake_case_42"],
            ),
        ];

        for (input, expected) in test_cases {
            let result = parse_instruction_symbols(input)
                .into_iter()
                .map(|s| format!("{s:?}"))
                .collect::<Vec<_>>();
            assert_eq!(result, expected, "Failed on input: {}", input);
        }
    }
}
