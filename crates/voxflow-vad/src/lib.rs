use thiserror::Error;
use webrtc_vad::{SampleRate, Vad, VadMode};

pub const SAMPLE_RATE: u32 = 16_000;
pub const FRAME_MS: u32 = 30;
pub const FRAME_SAMPLES: usize = (SAMPLE_RATE as usize * FRAME_MS as usize) / 1000;
const PRE_ROLL_FRAMES: usize = 5; // 150 ms
const POST_ROLL_FRAMES: usize = 10; // 300 ms

#[derive(Debug, Error)]
pub enum VadError {
    #[error("invalid frame size: expected {FRAME_SAMPLES} samples")]
    InvalidFrameSize,
}

pub struct VoiceActivityDetector {
    vad: Vad,
}

impl VoiceActivityDetector {
    pub fn new(aggressiveness: u8) -> Self {
        let mode = match aggressiveness {
            0 => VadMode::Quality,
            1 => VadMode::LowBitrate,
            2 => VadMode::Aggressive,
            _ => VadMode::VeryAggressive,
        };
        Self {
            vad: Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, mode),
        }
    }

    pub fn default_detector() -> Self {
        Self::new(2)
    }

    pub fn is_speech_frame(&mut self, frame: &[i16]) -> Result<bool, VadError> {
        if frame.len() != FRAME_SAMPLES {
            return Err(VadError::InvalidFrameSize);
        }
        self.vad
            .is_voice_segment(frame)
            .map_err(|_| VadError::InvalidFrameSize)
    }

    pub fn trim_silence(&mut self, pcm: &[i16]) -> Result<TrimResult, VadError> {
        if pcm.is_empty() {
            return Ok(TrimResult {
                trimmed: Vec::new(),
                raw_duration_secs: 0.0,
                trimmed_duration_secs: 0.0,
            });
        }

        let raw_duration_secs = pcm.len() as f32 / SAMPLE_RATE as f32;
        let mut speech_frame_indices = Vec::new();

        for (index, chunk) in pcm.chunks(FRAME_SAMPLES).enumerate() {
            if chunk.len() < FRAME_SAMPLES {
                break;
            }
            let speech = self.is_speech_frame(chunk)?;
            if speech {
                speech_frame_indices.push(index);
            }
        }

        let trimmed = contiguous_speech_span(pcm, &speech_frame_indices);

        Ok(TrimResult {
            trimmed_duration_secs: trimmed.len() as f32 / SAMPLE_RATE as f32,
            raw_duration_secs,
            trimmed,
        })
    }
}

/// Keep one continuous region around detected speech. Concatenating only the
/// positive VAD frames removes unvoiced consonants and destroys timing, which
/// is especially harmful for a single short word.
fn contiguous_speech_span(pcm: &[i16], speech_frames: &[usize]) -> Vec<i16> {
    let (Some(&first), Some(&last)) = (speech_frames.first(), speech_frames.last()) else {
        return pcm.to_vec();
    };
    let start = first.saturating_sub(PRE_ROLL_FRAMES) * FRAME_SAMPLES;
    let end = ((last + 1 + POST_ROLL_FRAMES) * FRAME_SAMPLES).min(pcm.len());
    pcm[start..end].to_vec()
}

#[derive(Debug, Clone)]
pub struct TrimResult {
    pub trimmed: Vec<i16>,
    pub raw_duration_secs: f32,
    pub trimmed_duration_secs: f32,
}

pub fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_empty() {
        let mut vad = VoiceActivityDetector::default_detector();
        let result = vad.trim_silence(&[]).unwrap();
        assert!(result.trimmed.is_empty());
    }

    #[test]
    fn trimming_preserves_contiguous_context_and_gaps() {
        let pcm: Vec<i16> = (0..30 * FRAME_SAMPLES).map(|i| i as i16).collect();
        let trimmed = contiguous_speech_span(&pcm, &[10, 12]);
        assert_eq!(trimmed.first(), pcm.get(5 * FRAME_SAMPLES));
        assert_eq!(trimmed.len(), 18 * FRAME_SAMPLES);
        assert!(trimmed.contains(&((11 * FRAME_SAMPLES) as i16)));
    }

    #[test]
    fn trailing_partial_frame_is_retained_inside_post_roll() {
        let pcm = vec![1_i16; 4 * FRAME_SAMPLES + 123];
        let trimmed = contiguous_speech_span(&pcm, &[3]);
        assert_eq!(trimmed.len(), pcm.len());
    }

    #[test]
    fn no_detected_speech_keeps_full_recording() {
        let pcm = vec![1_i16; 2 * FRAME_SAMPLES + 17];
        assert_eq!(contiguous_speech_span(&pcm, &[]), pcm);
    }
}
