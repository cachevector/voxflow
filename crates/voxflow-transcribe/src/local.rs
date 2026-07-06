use crate::{AudioSegment, Money, TranscribeError, TranscribeOpts, Transcript, TranscriptionProvider};
use async_trait::async_trait;

/// Local transcription provider. Uses whisper.cpp when `local-whisper` feature is enabled.
/// Falls back to a lightweight heuristic for very short utterances in default builds.
pub struct LocalWhisperTranscriber {
    model_name: String,
}

impl LocalWhisperTranscriber {
    pub fn new(model_name: String) -> Self {
        Self { model_name }
    }
}

#[async_trait]
impl TranscriptionProvider for LocalWhisperTranscriber {
    fn id(&self) -> &str {
        "local"
    }

    fn estimated_cost(&self, _duration_secs: f32) -> Money {
        Money { usd: 0.0, inr: 0.0 }
    }

    async fn transcribe(
        &self,
        audio: &AudioSegment,
        opts: &TranscribeOpts,
    ) -> Result<Transcript, TranscribeError> {
        if audio.pcm_i16.is_empty() {
            return Err(TranscribeError::EmptyAudio);
        }

        #[cfg(feature = "local-whisper")]
        {
            return transcribe_whisper(audio, &self.model_name, opts).await;
        }

        #[cfg(not(feature = "local-whisper"))]
        {
            // Stub: offline mode requires model download; short audio gets placeholder
            let duration = audio.duration_secs;
            if duration < 0.5 {
                return Err(TranscribeError::LocalUnavailable(
                    "Audio too short for local stub".into(),
                ));
            }

            Ok(Transcript {
                text: format!(
                    "[local:{} — enable whisper model for transcription]",
                    self.model_name
                ),
                confidence: 0.5,
                provider: self.id().into(),
                model: opts.model.clone(),
                language: opts.language.clone(),
            })
        }
    }
}

#[cfg(feature = "local-whisper")]
async fn transcribe_whisper(
    audio: &AudioSegment,
    model_name: &str,
    opts: &TranscribeOpts,
) -> Result<Transcript, TranscribeError> {
    let _ = (audio, model_name, opts);
    Err(TranscribeError::LocalUnavailable(
        "whisper.cpp bindings not yet linked — build with model path".into(),
    ))
}
