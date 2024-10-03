use std::time::Instant;

use anyhow::Context;
use ureq::{json, serde_json::Value};

const INSTRUCTIONS: &str =
"You are a code modification assistant. Your task is to precisely apply specified changes to given code structures while adhering to the following guidelines:

1. Make only the changes explicitly requested.
2. Preserve all existing code not mentioned in the change request.
3. Maintain all existing `...` placeholders without modification.
4. Do not introduce any new code, comments, or placeholders unless specifically requested.
5. Output the entire updated file structure.
6. Provide only the modified code without explanations or questions.
7. Follow the edits carefully, ensuring all requested changes are implemented.
8. Merge edits into existing items if possible.
9. When adding new methods or implementations, place them in the appropriate location within the existing structure.
10. Pay close attention to the placement of new structs, traits, and implementations, inserting them logically within the file.
11. Ensure that all new additions are included in the output, even if they require creating new sections in the file.

Your response should consist solely of the updated code structure.";

pub fn prompt_for_edits(
    language: &str,
    collapsed_document: &str,
    patch: &str,
) -> anyhow::Result<String> {
    let start = Instant::now();
    let prompt = format!(
        "Given the following file structure:

```{language}
{collapsed_document}
```

Make the follow edits:
{patch}"
    );
    let api_key = std::env::var("SAMBANOVA_API_KEY").expect("SAMBANOVA_API_KEY must be set");
    // TODO: check if streaming faster
    let response = ureq::post("https://api.sambanova.ai/v1/chat/completions")
        .set("Authorization", &format!("Bearer {}", api_key))
        .send_json(json!({
            "model": "Meta-Llama-3.1-70B-Instruct",
            "messages": [
                { "role": "system", "content": INSTRUCTIONS },
                { "role": "user", "content": prompt }
            ],
            "temperature": 0.0,
        }))?;
    let value = response.into_json::<Value>()?;
    let content = value["choices"][0]["message"]["content"]
        .as_str()
        .context("invalid output")?;
    let trimmed = if content.starts_with("```") {
        let start = content
            .find("\n")
            .map(|x| x + "\n".len())
            .unwrap_or("```".len());
        &content[start..]
    } else {
        content
    };

    Ok(trimmed.trim_end_matches("\n```").to_owned())
}
