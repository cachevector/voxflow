//! Diagnostic: opens the default input device, records for a few seconds and
//! reports signal level. macOS hands back digital silence (rather than an
//! error) when Microphone permission is missing, so a near-zero peak here means
//! a permission/device problem rather than a pipeline bug.

use anyhow::Result;
use voxflow_audio::AudioCapture;

fn main() -> Result<()> {
    println!("input devices:");
    for d in AudioCapture::list_devices()? {
        println!("  {}{}", d.name, if d.is_default { "  (default)" } else { "" });
    }

    let capture = AudioCapture::open(None)?;
    println!(
        "\nopened default device at {} Hz — speak now, recording 4s…",
        capture.sample_rate()
    );

    let _ = capture.drain_samples();
    std::thread::sleep(std::time::Duration::from_secs(4));
    let samples = capture.drain_samples();

    let peak = samples.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
    let rms = if samples.is_empty() {
        0.0
    } else {
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
    };

    println!("\nsamples: {}", samples.len());
    println!("peak:    {peak:.6}");
    println!("rms:     {rms:.6}");

    // Speech into a correctly-configured mic peaks well above 0.05. Anything
    // under that is room noise, which Whisper transcribes as "[BLANK_AUDIO]".
    if samples.is_empty() {
        println!("\nVERDICT: no samples captured — device did not deliver any audio.");
    } else if peak < 1e-6 {
        println!("\nVERDICT: digital silence (all zeros) — Microphone permission is denied.");
    } else if peak < 0.05 {
        println!(
            "\nVERDICT: signal far too quiet for speech (peak {peak:.4}).\n\
             Permission is granted, but the mic is muted, gained down, or you spoke\n\
             into a different device than the default shown above."
        );
    } else {
        println!("\nVERDICT: healthy speech level — the microphone path works.");
    }

    Ok(())
}
