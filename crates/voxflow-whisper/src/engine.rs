use crate::model::{ensure_model_downloaded, model_path_for_id};
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use thiserror::Error;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[derive(Debug, Error)]
pub enum WhisperError {
    #[error("model: {0}")]
    Model(String),
    #[error("inference: {0}")]
    Inference(String),
    #[error("empty audio")]
    EmptyAudio,
}

pub struct WhisperEngine {
    model_id: String,
    context: Mutex<Option<WhisperContext>>,
    loaded_path: Mutex<Option<PathBuf>>,
}

impl WhisperEngine {
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            context: Mutex::new(None),
            loaded_path: Mutex::new(None),
        }
    }

    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    pub fn is_loaded(&self) -> bool {
        self.context.lock().is_some()
    }

    pub async fn prewarm(&self) -> Result<(), WhisperError> {
        let path = ensure_model_downloaded(&self.model_id)
            .await
            .map_err(|e| WhisperError::Model(e.to_string()))?;
        self.load_from_path(&path)
    }

    fn load_from_path(&self, path: &Path) -> Result<(), WhisperError> {
        let mut ctx_guard = self.context.lock();
        if ctx_guard.is_some() {
            return Ok(());
        }
        let ctx = WhisperContext::new_with_params(
            path.to_str()
                .ok_or_else(|| WhisperError::Model("invalid model path".into()))?,
            WhisperContextParameters::default(),
        )
        .map_err(|e| WhisperError::Model(e.to_string()))?;
        *ctx_guard = Some(ctx);
        *self.loaded_path.lock() = Some(path.to_path_buf());
        Ok(())
    }

    pub async fn transcribe_pcm_i16(
        &self,
        pcm_i16: &[i16],
        sample_rate: u32,
    ) -> Result<String, WhisperError> {
        if pcm_i16.is_empty() {
            return Err(WhisperError::EmptyAudio);
        }

        let path = if self.context.lock().is_none() {
            ensure_model_downloaded(&self.model_id)
                .await
                .map_err(|e| WhisperError::Model(e.to_string()))?
        } else {
            model_path_for_id(&self.model_id).map_err(|e| WhisperError::Model(e.to_string()))?
        };
        self.load_from_path(&path)?;

        let audio_f32: Vec<f32> = if sample_rate == 16_000 {
            pcm_i16.iter().map(|&s| s as f32 / 32768.0).collect()
        } else {
            resample_to_16k(pcm_i16, sample_rate)
        };

        let ctx = self.context.lock();
        let ctx = ctx
            .as_ref()
            .ok_or_else(|| WhisperError::Model("context not loaded".into()))?;

        let mut state = ctx
            .create_state()
            .map_err(|e| WhisperError::Inference(e.to_string()))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(num_cpus());
        params.set_translate(false);
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state
            .full(params, &audio_f32)
            .map_err(|e| WhisperError::Inference(e.to_string()))?;

        let n = state
            .full_n_segments()
            .map_err(|e| WhisperError::Inference(e.to_string()))?;
        // Whisper emits one segment per utterance chunk, each normally carrying a
        // leading space (" Hello there." / " How are you?"). Each segment is
        // trimmed and rejoined with exactly one space — trimming without
        // re-adding the separator is what used to glue sentences together as
        // "Hello there.How are you?".
        let mut text = String::new();
        for i in 0..n {
            if let Ok(segment) = state.full_get_segment_text(i) {
                let segment = segment.trim();
                if segment.is_empty() {
                    continue;
                }
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(segment);
            }
        }
        Ok(strip_non_speech_markers(text.trim()))
    }
}

/// Parenthesised sound events Whisper emits when it hears no speech.
const NON_SPEECH_HINTS: &[&str] = &[
    "blank_audio",
    "silence",
    "music",
    "applause",
    "laughter",
    "laughs",
    "inaudible",
    "noise",
    "beep",
    "static",
    "coughs",
    "sighs",
    "clears throat",
];

/// Whisper annotates non-speech audio inline — `[BLANK_AUDIO]`, `[ Silence ]`,
/// `(upbeat music)`. Pasting those into the user's document is never correct.
///
/// Square brackets are always annotations: dictated speech does not produce
/// them. Parentheses are only dropped when the content looks like a sound
/// event, since "(see figure 3)" is plausible dictation.
pub fn strip_non_speech_markers(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(start) = rest.find(['[', '(']) {
        let open = rest.as_bytes()[start] as char;
        let close = if open == '[' { ']' } else { ')' };

        let Some(end) = rest[start..].find(close).map(|i| start + i) else {
            break;
        };

        let inner = rest[start + 1..end].trim().to_lowercase();
        let drop = open == '['
            || NON_SPEECH_HINTS
                .iter()
                .any(|hint| inner.contains(hint));

        out.push_str(&rest[..start]);
        if !drop {
            out.push_str(&rest[start..=end]);
        }
        rest = &rest[end + 1..];
    }
    out.push_str(rest);

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod marker_tests {
    use super::strip_non_speech_markers;

    #[test]
    fn drops_blank_audio_sentinel() {
        assert_eq!(strip_non_speech_markers("[BLANK_AUDIO]"), "");
        assert_eq!(strip_non_speech_markers("[ Silence ]"), "");
    }

    #[test]
    fn drops_markers_mixed_with_speech() {
        assert_eq!(
            strip_non_speech_markers("[BLANK_AUDIO] hello there [MUSIC]"),
            "hello there"
        );
        assert_eq!(strip_non_speech_markers("hi (upbeat music) there"), "hi there");
    }

    #[test]
    fn keeps_ordinary_parenthetical_speech() {
        assert_eq!(
            strip_non_speech_markers("check the value (see figure 3) first"),
            "check the value (see figure 3) first"
        );
    }

    #[test]
    fn joined_segments_keep_sentence_spacing() {
        // Mirrors the segment-join loop in transcribe_pcm_i16: Whisper hands back
        // segments with leading spaces, and they must not be glued together.
        let segments = [" Hello there.", " How are you?"];
        let mut text = String::new();
        for segment in segments {
            let segment = segment.trim();
            if segment.is_empty() {
                continue;
            }
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(segment);
        }
        assert_eq!(text, "Hello there. How are you?");
    }

    #[test]
    fn leaves_plain_speech_untouched() {
        assert_eq!(
            strip_non_speech_markers("let's try again"),
            "let's try again"
        );
    }

    #[test]
    fn tolerates_unclosed_bracket() {
        assert_eq!(strip_non_speech_markers("hello [oops"), "hello [oops");
    }
}

fn num_cpus() -> i32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(4)
        .clamp(1, 8)
}

fn resample_to_16k(pcm_i16: &[i16], sample_rate: u32) -> Vec<f32> {
    if sample_rate == 0 {
        return Vec::new();
    }
    let ratio = 16_000.0 / sample_rate as f64;
    let out_len = ((pcm_i16.len() as f64) * ratio).ceil() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = (i as f64 / ratio) as usize;
        let sample = pcm_i16.get(src).copied().unwrap_or(0);
        out.push(sample as f32 / 32768.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_identity_at_16k() {
        let pcm = vec![0_i16, 1000, -1000, 500];
        let f32 = resample_to_16k(&pcm, 16_000);
        assert_eq!(f32.len(), 4);
    }
}
