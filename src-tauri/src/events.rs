use tauri::{AppHandle, Emitter};
use voxflow_core::StateEvent;

pub const DICTATION_STATE: &str = "dictation://state";
pub const DICTATION_AMPLITUDE: &str = "dictation://amplitude";
pub const VOCABULARY_SUGGESTION: &str = "dictation://vocabulary-suggestion";
pub const VOCABULARY_SUGGESTION_CLEARED: &str = "dictation://vocabulary-suggestion-cleared";

pub fn emit_state(app: &AppHandle, event: &StateEvent) {
    if let Err(e) = app.emit(DICTATION_STATE, event) {
        tracing::warn!("failed to emit dictation state event: {e}");
    }
}

// Streamed at ~30fps while listening (see hotkey_listener::spawn_amplitude_poller)
// to drive the live overlay waveform.
pub fn emit_amplitude(app: &AppHandle, level: f32) {
    if let Err(e) = app.emit(DICTATION_AMPLITUDE, level) {
        tracing::warn!("failed to emit amplitude event: {e}");
    }
}

pub fn emit_vocabulary_suggestion(
    app: &AppHandle,
    suggestion: &voxflow_core::VocabularySuggestion,
) {
    if let Err(e) = app.emit(VOCABULARY_SUGGESTION, suggestion) {
        tracing::warn!("failed to emit vocabulary suggestion: {e}");
    }
}

pub fn emit_vocabulary_suggestion_cleared(app: &AppHandle) {
    if let Err(e) = app.emit(VOCABULARY_SUGGESTION_CLEARED, ()) {
        tracing::warn!("failed to emit vocabulary suggestion clear event: {e}");
    }
}
