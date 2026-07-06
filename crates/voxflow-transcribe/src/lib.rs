use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod cleanup;
pub mod groq;
pub mod local;
pub mod openai;

pub use cleanup::{apply_dictionary, apply_snippets, CleanupEngine};
pub use groq::{DeepgramTranscriber, GroqTranscriber};
pub use local::LocalWhisperTranscriber;
pub use openai::OpenAiTranscriber;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSegment {
    pub pcm_i16: Vec<i16>,
    pub sample_rate: u32,
    pub duration_secs: f32,
}

impl AudioSegment {
    pub fn to_wav_bytes(&self) -> Result<Vec<u8>, TranscribeError> {
        let mut cursor = std::io::Cursor::new(Vec::new());
        {
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: self.sample_rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut writer = hound::WavWriter::new(&mut cursor, spec)
                .map_err(|e| TranscribeError::Encode(e.to_string()))?;
            for sample in &self.pcm_i16 {
                writer
                    .write_sample(*sample)
                    .map_err(|e| TranscribeError::Encode(e.to_string()))?;
            }
            writer
                .finalize()
                .map_err(|e| TranscribeError::Encode(e.to_string()))?;
        }
        Ok(cursor.into_inner())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub text: String,
    pub confidence: f32,
    pub provider: String,
    pub model: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscribeOpts {
    pub model: String,
    pub language: Option<String>,
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    pub usd: f32,
    pub inr: f32,
}

#[derive(Debug, Error)]
pub enum TranscribeError {
    #[error("network: {0}")]
    Network(String),
    #[error("api: {0}")]
    Api(String),
    #[error("encode: {0}")]
    Encode(String),
    #[error("local model unavailable: {0}")]
    LocalUnavailable(String),
    #[error("empty audio")]
    EmptyAudio,
}

#[async_trait]
pub trait TranscriptionProvider: Send + Sync {
    fn id(&self) -> &str;
    fn supports_streaming(&self) -> bool {
        false
    }
    fn estimated_cost(&self, duration_secs: f32) -> Money;
    async fn transcribe(
        &self,
        audio: &AudioSegment,
        opts: &TranscribeOpts,
    ) -> Result<Transcript, TranscribeError>;
}
