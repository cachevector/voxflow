use async_trait::async_trait;
use crate::{AudioSegment, Money, TranscribeError, TranscribeOpts, Transcript, TranscriptionProvider};
use voxflow_cost::{estimate_openai_cost_usd, usd_to_inr};

pub struct GroqTranscriber {
    api_key: String,
}

impl GroqTranscriber {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl TranscriptionProvider for GroqTranscriber {
    fn id(&self) -> &str {
        "groq"
    }

    fn estimated_cost(&self, duration_secs: f32) -> Money {
        let usd = estimate_openai_cost_usd("gpt-4o-mini-transcribe", duration_secs) * 0.5;
        Money { usd, inr: usd_to_inr(usd) }
    }

    async fn transcribe(
        &self,
        _audio: &AudioSegment,
        opts: &TranscribeOpts,
    ) -> Result<Transcript, TranscribeError> {
        let _ = (&self.api_key, opts);
        Err(TranscribeError::LocalUnavailable(
            "Groq provider adapter stub — configure endpoint in Pro settings".into(),
        ))
    }
}

pub struct DeepgramTranscriber {
    api_key: String,
}

impl DeepgramTranscriber {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl TranscriptionProvider for DeepgramTranscriber {
    fn id(&self) -> &str {
        "deepgram"
    }

    fn estimated_cost(&self, duration_secs: f32) -> Money {
        let usd = estimate_openai_cost_usd("gpt-4o-mini-transcribe", duration_secs) * 0.8;
        Money { usd, inr: usd_to_inr(usd) }
    }

    async fn transcribe(
        &self,
        _audio: &AudioSegment,
        opts: &TranscribeOpts,
    ) -> Result<Transcript, TranscribeError> {
        let _ = (&self.api_key, opts);
        Err(TranscribeError::LocalUnavailable(
            "Deepgram provider adapter stub — Pro feature".into(),
        ))
    }
}
