//! One generic OpenAI-API-compatible adapter used for both the transcription
//! call and the AI rewrite call. The same code path works against OpenAI,
//! Groq, or a fully self-hosted server (e.g. whisper.cpp-server/llama.cpp-server
//! running on a Raspberry Pi) — only the base URL and key differ.

mod client;
pub mod rewrite;

pub use client::{OpenAiCompatibleClient, ProviderError};
pub use rewrite::{
    apply_dictionary, apply_rules, apply_snippets, finalize_text, system_prompt_for_mode,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSegment {
    pub pcm_i16: Vec<i16>,
    pub sample_rate: u32,
    pub duration_secs: f32,
}

impl AudioSegment {
    pub fn to_wav_bytes(&self) -> Result<Vec<u8>, ProviderError> {
        let mut cursor = std::io::Cursor::new(Vec::new());
        {
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: self.sample_rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut writer = hound::WavWriter::new(&mut cursor, spec)
                .map_err(|e| ProviderError::Encode(e.to_string()))?;
            for sample in &self.pcm_i16 {
                writer
                    .write_sample(*sample)
                    .map_err(|e| ProviderError::Encode(e.to_string()))?;
            }
            writer
                .finalize()
                .map_err(|e| ProviderError::Encode(e.to_string()))?;
        }
        Ok(cursor.into_inner())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub text: String,
    pub provider: String,
    pub model: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranscribeOpts {
    pub model: String,
    pub language: Option<String>,
    pub prompt: Option<String>,
}

/// Builds a client from a `ProviderConfig` plus its already-resolved secret
/// (looked up from `voxflow-secrets` by the caller, keeping this crate free
/// of a dependency on keyring/OS APIs).
pub fn build_client(
    config: &voxflow_config::ProviderConfig,
    api_key: Option<String>,
) -> OpenAiCompatibleClient {
    OpenAiCompatibleClient::new(config.resolved_base_url(), api_key)
}
