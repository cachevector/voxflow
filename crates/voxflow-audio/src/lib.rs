use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use thiserror::Error;
use tracing::warn;

pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Audio callbacks can still be queued briefly after the hotkey release is
/// observed. Keep the stream alive for this long before draining so the final
/// syllables make it into the utterance.
pub const CAPTURE_FLUSH_GRACE: std::time::Duration = std::time::Duration::from_millis(150);

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("no input device available")]
    NoInputDevice,
    #[error("no supported config for device")]
    NoSupportedConfig,
    #[error("stream error: {0}")]
    Stream(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

enum AudioCommand {
    Shutdown,
}

/// Send+Sync handle to a background audio capture thread.
pub struct AudioCapture {
    buffer: Arc<Mutex<Vec<f32>>>,
    healthy: Arc<AtomicBool>,
    sample_rate: u32,
    _thread: JoinHandle<()>,
    cmd_tx: std::sync::mpsc::Sender<AudioCommand>,
}

impl AudioCapture {
    pub fn list_devices() -> Result<Vec<AudioDeviceInfo>> {
        let host = cpal::default_host();
        let default_name = host.default_input_device().and_then(|d| d.name().ok());

        let devices: Vec<AudioDeviceInfo> = host
            .input_devices()
            .context("failed to enumerate input devices")?
            .filter_map(|d| {
                let name = d.name().ok()?;
                let id = name.clone();
                let is_default = default_name.as_ref() == Some(&name);
                Some(AudioDeviceInfo {
                    id,
                    name,
                    is_default,
                })
            })
            .collect();
        Ok(devices)
    }

    pub fn open(device_id: Option<&str>) -> Result<Self> {
        let host = cpal::default_host();
        let device = if let Some(id) = device_id {
            host.input_devices()
                .context("enumerate devices")?
                .find(|d| d.name().map(|n| n == id).unwrap_or(false))
                .context("device not found")?
        } else {
            host.default_input_device()
                .context(AudioError::NoInputDevice)?
        };

        let config = device
            .default_input_config()
            .context(AudioError::NoSupportedConfig)?;
        let sample_rate = config.sample_rate().0;
        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();

        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let buffer_clone = buffer.clone();
        let healthy = Arc::new(AtomicBool::new(false));
        let healthy_clone = healthy.clone();
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);

        let thread = thread::spawn(move || {
            let stream = build_stream(
                &device,
                &stream_config,
                sample_format,
                buffer_clone,
                healthy_clone.clone(),
            );
            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    warn!("failed to build audio stream: {e}");
                    let _ = ready_tx.send(Err(e.to_string()));
                    return;
                }
            };
            healthy_clone.store(true, Ordering::SeqCst);
            if let Err(e) = stream.play() {
                healthy_clone.store(false, Ordering::SeqCst);
                warn!("failed to play stream: {e}");
                let _ = ready_tx.send(Err(e.to_string()));
                return;
            }
            let _ = ready_tx.send(Ok(()));
            loop {
                match cmd_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(AudioCommand::Shutdown)
                    | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                }
            }
            healthy_clone.store(false, Ordering::SeqCst);
            drop(stream);
        });

        match ready_rx.recv_timeout(std::time::Duration::from_secs(2)) {
            Ok(Ok(())) => {}
            Ok(Err(error)) => return Err(AudioError::Stream(error).into()),
            Err(error) => {
                return Err(
                    AudioError::Stream(format!("timed out starting input stream: {error}")).into(),
                )
            }
        }

        Ok(Self {
            buffer,
            healthy,
            sample_rate,
            _thread: thread,
            cmd_tx,
        })
    }

    pub fn prewarm(device_id: Option<&str>) -> Result<Self> {
        Self::open(device_id)
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::SeqCst)
    }

    /// Peak-normalized loudness of the most recently captured audio, without
    /// draining the buffer. Drives the live overlay waveform while listening.
    ///
    /// Returns roughly 0.0 (silence) .. 1.0 (loud speech). We use peak rather
    /// than pure RMS so brief consonants and transients still visibly move the
    /// wave, then apply a gain so normal speaking levels fill most of the range.
    pub fn current_level(&self) -> f32 {
        const WINDOW: usize = 2_048; // ~40ms at 48kHz — a responsive meter window.
        const GAIN: f32 = 3.5;

        let buf = self.buffer.lock();
        if buf.is_empty() {
            return 0.0;
        }
        let start = buf.len().saturating_sub(WINDOW);
        let window = &buf[start..];

        let mut sum_sq = 0.0f32;
        let mut peak = 0.0f32;
        for &s in window {
            sum_sq += s * s;
            peak = peak.max(s.abs());
        }
        let rms = (sum_sq / window.len() as f32).sqrt();

        // Blend RMS (body) with peak (transients) for a lively but stable meter.
        let level = (0.6 * rms + 0.4 * peak) * GAIN;
        level.clamp(0.0, 1.0)
    }

    pub fn drain_samples(&self) -> Vec<f32> {
        let mut buf = self.buffer.lock();
        let samples = buf.clone();
        buf.clear();
        samples
    }

    pub fn resample_to_16k(&self, samples: &[f32]) -> Vec<f32> {
        if self.sample_rate == TARGET_SAMPLE_RATE {
            return samples.to_vec();
        }
        let ratio = TARGET_SAMPLE_RATE as f64 / self.sample_rate as f64;
        let out_len = (samples.len() as f64 * ratio).ceil() as usize;
        let mut out = Vec::with_capacity(out_len);
        for i in 0..out_len {
            let src_idx = (i as f64 / ratio) as usize;
            let sample = samples.get(src_idx).copied().unwrap_or(0.0);
            out.push(sample);
        }
        out
    }

    pub fn take_utterance_pcm_i16(&self) -> Vec<i16> {
        let samples = self.drain_samples();
        let resampled = self.resample_to_16k(&samples);
        resampled
            .iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .collect()
    }

    /// Finalize an utterance after allowing in-flight input callbacks to
    /// append their last samples. This is intentionally separate from
    /// `take_utterance_pcm_i16` so callers that need an immediate snapshot do
    /// not inherit a timing delay.
    pub fn take_utterance_pcm_i16_after_flush(&self) -> Vec<i16> {
        thread::sleep(CAPTURE_FLUSH_GRACE);
        self.take_utterance_pcm_i16()
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(AudioCommand::Shutdown);
    }
}

fn build_stream(
    device: &Device,
    stream_config: &StreamConfig,
    sample_format: SampleFormat,
    buffer: Arc<Mutex<Vec<f32>>>,
    healthy: Arc<AtomicBool>,
) -> Result<Stream> {
    match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            stream_config,
            move |data: &[f32], _| buffer.lock().extend_from_slice(data),
            move |err| {
                healthy.store(false, Ordering::SeqCst);
                warn!("audio stream error: {err}");
            },
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            stream_config,
            move |data: &[i16], _| {
                let converted: Vec<f32> =
                    data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                buffer.lock().extend_from_slice(&converted);
            },
            move |err| {
                healthy.store(false, Ordering::SeqCst);
                warn!("audio stream error: {err}");
            },
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            stream_config,
            move |data: &[u16], _| {
                let converted: Vec<f32> = data
                    .iter()
                    .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                    .collect();
                buffer.lock().extend_from_slice(&converted);
            },
            move |err| {
                healthy.store(false, Ordering::SeqCst);
                warn!("audio stream error: {err}");
            },
            None,
        ),
        _ => anyhow::bail!("unsupported sample format"),
    }
    .context("build stream")
}

#[cfg(test)]
mod tests {
    #[test]
    fn resample_ratio() {
        assert_eq!(16000u32, super::TARGET_SAMPLE_RATE);
    }
}
