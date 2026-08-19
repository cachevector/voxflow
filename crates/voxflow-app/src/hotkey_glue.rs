use crate::app_state::{uses_toggle_mode, AppState};
use std::sync::Arc;
use voxflow_core::{DictationState, StateEvent};
use voxflow_platform::HotkeyEvent;

/// Bridges hotkey press/release/toggle events onto `DictationEngine` calls,
/// and forwards resulting `StateEvent`s to `overlay_tx` so the GPUI overlay
/// can react on the main thread. Runs on its own OS thread since
/// `DictationEngine::on_hotkey_down`/`on_hotkey_up` are synchronous, blocking
/// calls (the latter runs the whole VAD -> transcribe -> cleanup -> insert
/// pipeline) and must never run on GPUI's main/foreground thread.
pub fn spawn(state: Arc<AppState>, overlay_tx: async_channel::Sender<StateEvent>) {
    let toggle_mode = uses_toggle_mode(&state.session);
    let events = state.hotkey.events();
    let engine = state.engine.clone();

    std::thread::spawn(move || {
        let mut listening = false;
        while let Ok(event) = events.recv_blocking() {
            let should_start = match (toggle_mode, event, listening) {
                (false, HotkeyEvent::Pressed, false) => true,
                (false, HotkeyEvent::Released, true) => false,
                (true, HotkeyEvent::Toggled, was_listening) => !was_listening,
                // Ignore events that don't match this backend's expected
                // shape (e.g. a stray Toggled from a backend that should
                // only emit Pressed/Released, or a redundant Pressed while
                // already listening).
                _ => continue,
            };

            if should_start {
                let event = engine.on_hotkey_down();
                listening = true;
                let _ = overlay_tx.send_blocking(event);
            } else {
                listening = false;
                let event = match engine.on_hotkey_up(None) {
                    Ok(result) => StateEvent {
                        session_id: 0,
                        ui_state: result.state.into(),
                        state: result.state,
                        message: None,
                        transcript_preview: Some(result.text),
                    },
                    Err(e) => {
                        tracing::warn!(error = %e, "dictation pipeline failed");
                        StateEvent::new(DictationState::Error, Some(e.to_string()))
                    }
                };
                let _ = overlay_tx.send_blocking(event);
            }
        }
    });
}
