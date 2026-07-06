use crate::{AudioSegment, Money, TranscribeError, TranscribeOpts, Transcript, TranscriptionProvider};
use async_trait::async_trait;
use reqwest::Client;
use voxflow_cost::{estimate_openai_cost_usd, usd_to_inr};

pub struct OpenAiTranscriber {
    client: Client,
    api_key: String,
}

impl OpenAiTranscriber {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }
}

#[async_trait]
impl TranscriptionProvider for OpenAiTranscriber {
    fn id(&self) -> &str {
        "openai"
    }

    fn estimated_cost(&self, duration_secs: f32) -> Money {
        let usd = estimate_openai_cost_usd("gpt-4o-mini-transcribe", duration_secs);
        Money {
            usd,
            inr: usd_to_inr(usd),
        }
    }

    async fn transcribe(
        &self,
        audio: &AudioSegment,
        opts: &TranscribeOpts,
    ) -> Result<Transcript, TranscribeError> {
        if audio.pcm_i16.is_empty() {
            return Err(TranscribeError::EmptyAudio);
        }

        let wav = audio.to_wav_bytes()?;
        let part = reqwest::multipart::Part::bytes(wav)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| TranscribeError::Encode(e.to_string()))?;

        let mut form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", opts.model.clone());

        if let Some(lang) = &opts.language {
            form = form.text("language", lang.clone());
        }
        if let Some(prompt) = &opts.prompt {
            form = form.text("prompt", prompt.clone());
        }

        let response = self
            .client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| TranscribeError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TranscribeError::Api(body));
        }

        #[derive(serde::Deserialize)]
        struct ApiResponse {
            text: String,
        }

        let parsed: ApiResponse = response
            .json()
            .await
            .map_err(|e| TranscribeError::Api(e.to_string()))?;

        Ok(Transcript {
            text: parsed.text.trim().to_string(),
            confidence: 0.9,
            provider: self.id().into(),
            model: opts.model.clone(),
            language: opts.language.clone(),
        })
    }
}
