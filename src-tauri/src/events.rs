use tauri::{AppHandle, Emitter};
use voxflow_core::StateEvent;

pub const DICTATION_STATE: &str = "dictation://state";
pub const DICTATION_AMPLITUDE: &str = "dictation://amplitude";

pub fn emit_state(app: &AppHandle, event: &StateEvent) {
    if let Err(e) = app.emit(DICTATION_STATE, event) {
        tracing::warn!("failed to emit dictation state event: {e}");
    }
}

// Not called yet — the audio capture thread doesn't stream RMS samples out
// to the pipeline yet. Wired up in Phase 1 alongside the overlay waveform.
#[allow(dead_code)]
pub fn emit_amplitude(app: &AppHandle, level: f32) {
    if let Err(e) = app.emit(DICTATION_AMPLITUDE, level) {
        tracing::warn!("failed to emit amplitude event: {e}");
    }
}
