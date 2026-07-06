use thiserror::Error;
use webrtc_vad::{SampleRate, Vad, VadMode};

pub const SAMPLE_RATE: u32 = 16_000;
pub const FRAME_MS: u32 = 30;
pub const FRAME_SAMPLES: usize = (SAMPLE_RATE as usize * FRAME_MS as usize) / 1000;

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
        let mut speech_frames = Vec::new();
        let mut any_speech = false;

        for chunk in pcm.chunks(FRAME_SAMPLES) {
            if chunk.len() < FRAME_SAMPLES {
                break;
            }
            let speech = self.is_speech_frame(chunk)?;
            if speech {
                any_speech = true;
                speech_frames.extend_from_slice(chunk);
            }
        }

        let trimmed = if any_speech {
            speech_frames
        } else {
            pcm.to_vec()
        };

        Ok(TrimResult {
            trimmed_duration_secs: trimmed.len() as f32 / SAMPLE_RATE as f32,
            raw_duration_secs,
            trimmed,
        })
    }
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
}
