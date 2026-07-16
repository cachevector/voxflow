use crate::{AudioSegment, TranscribeOpts, Transcript};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("network: {0}")]
    Network(String),
    #[error("api: {0}")]
    Api(String),
    #[error("encode: {0}")]
    Encode(String),
    #[error("empty audio")]
    EmptyAudio,
}

/// A single OpenAI-API-compatible endpoint. Works unmodified against OpenAI,
/// Groq (`https://api.groq.com/openai/v1`), or a self-hosted server exposing
/// the same `/audio/transcriptions` and `/chat/completions` routes.
pub struct OpenAiCompatibleClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl OpenAiCompatibleClient {
    pub fn new(base_url: impl Into<String>, api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            api_key,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.base_url.trim_end_matches('/'))
    }

    pub async fn transcribe(
        &self,
        audio: &AudioSegment,
        opts: &TranscribeOpts,
    ) -> Result<Transcript, ProviderError> {
        if audio.pcm_i16.is_empty() {
            return Err(ProviderError::EmptyAudio);
        }

        let wav = audio.to_wav_bytes()?;
        let part = reqwest::multipart::Part::bytes(wav)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| ProviderError::Encode(e.to_string()))?;

        let mut form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", opts.model.clone());

        if let Some(lang) = &opts.language {
            form = form.text("language", lang.clone());
        }
        if let Some(prompt) = &opts.prompt {
            form = form.text("prompt", prompt.clone());
        }

        let mut request = self
            .client
            .post(self.url("audio/transcriptions"))
            .multipart(form);
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::Api(body));
        }

        #[derive(Deserialize)]
        struct ApiResponse {
            text: String,
        }

        let parsed: ApiResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::Api(e.to_string()))?;

        Ok(Transcript {
            text: parsed.text.trim().to_string(),
            provider: self.base_url.clone(),
            model: opts.model.clone(),
            language: opts.language.clone(),
        })
    }

    pub async fn chat_rewrite(
        &self,
        transcript: &str,
        system_prompt: &str,
        model: &str,
    ) -> Result<String, ProviderError> {
        if transcript.trim().is_empty() {
            return Ok(String::new());
        }

        #[derive(Serialize)]
        struct ChatMessage {
            role: String,
            content: String,
        }

        #[derive(Serialize)]
        struct Request {
            model: String,
            messages: Vec<ChatMessage>,
            max_tokens: u32,
        }

        let body = Request {
            model: model.to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: system_prompt.into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: transcript.into(),
                },
            ],
            max_tokens: 1024,
        };

        let mut request = self.client.post(self.url("chat/completions")).json(&body);
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::Api(body));
        }

        #[derive(Deserialize)]
        struct Choice {
            message: Message,
        }
        #[derive(Deserialize)]
        struct Message {
            content: String,
        }
        #[derive(Deserialize)]
        struct ApiResponse {
            choices: Vec<Choice>,
        }

        let parsed: ApiResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::Api(e.to_string()))?;

        Ok(parsed
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .unwrap_or_else(|| transcript.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_joins_base_and_path() {
        let client = OpenAiCompatibleClient::new("https://api.openai.com/v1/", None);
        assert_eq!(
            client.url("audio/transcriptions"),
            "https://api.openai.com/v1/audio/transcriptions"
        );
    }
}
