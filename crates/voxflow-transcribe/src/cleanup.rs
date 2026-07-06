use crate::TranscribeError;
use reqwest::Client;

pub struct CleanupEngine {
    client: Client,
    api_key: String,
}

impl CleanupEngine {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn cleanup(&self, text: &str, prompt: &str) -> Result<String, TranscribeError> {
        if text.trim().is_empty() {
            return Ok(String::new());
        }

        #[derive(serde::Serialize)]
        struct ChatMessage {
            role: String,
            content: String,
        }

        #[derive(serde::Serialize)]
        struct Request {
            model: String,
            messages: Vec<ChatMessage>,
            max_tokens: u32,
        }

        let body = Request {
            model: "gpt-4o-mini".into(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: prompt.into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: text.into(),
                },
            ],
            max_tokens: 1024,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| TranscribeError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let err = response.text().await.unwrap_or_default();
            return Err(TranscribeError::Api(err));
        }

        #[derive(serde::Deserialize)]
        struct Choice {
            message: Message,
        }
        #[derive(serde::Deserialize)]
        struct Message {
            content: String,
        }
        #[derive(serde::Deserialize)]
        struct ApiResponse {
            choices: Vec<Choice>,
        }

        let parsed: ApiResponse = response
            .json()
            .await
            .map_err(|e| TranscribeError::Api(e.to_string()))?;

        Ok(parsed
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .unwrap_or_else(|| text.to_string()))
    }

    pub fn apply_rules(text: &str, mode: voxflow_config::OutputMode) -> String {
        match mode {
            voxflow_config::OutputMode::PlainText | voxflow_config::OutputMode::TerminalSafe => {
                text.replace(['“', '”'], "\"").replace(['‘', '’'], "'")
            }
            voxflow_config::OutputMode::Casual => text.trim().to_string(),
            voxflow_config::OutputMode::CodePreserve => text.to_string(),
            _ => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    return String::new();
                }
                let mut chars = trimmed.chars();
                let first = chars.next().unwrap().to_uppercase().collect::<String>();
                format!("{first}{}", chars.as_str())
            }
        }
    }
}

pub fn apply_snippets(text: &str, snippets: &[voxflow_config::Snippet]) -> String {
    let mut result = text.to_string();
    for snippet in snippets {
        if result.contains(&snippet.trigger) {
            result = result.replace(&snippet.trigger, &snippet.expansion);
        }
    }
    result
}

pub fn apply_dictionary(text: &str, dictionary: &[voxflow_config::DictionaryEntry]) -> String {
    let mut result = text.to_string();
    for entry in dictionary {
        if let Some(replacement) = &entry.replacement {
            result = result.replace(&entry.term, replacement);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_safe_quotes() {
        let out = CleanupEngine::apply_rules("“hello”", voxflow_config::OutputMode::TerminalSafe);
        assert!(out.contains('"'));
    }
}
