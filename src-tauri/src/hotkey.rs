use crate::state::AppState;
use crate::{events, windows};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

/// Default binding: a modifier+key combo (not a bare modifier), so no
/// Input Monitoring permission is needed on macOS for the common case.
pub fn default_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Space)
}

pub fn handle_shortcut(app: &AppHandle, state: ShortcutState) {
    let app_state = app.state::<AppState>();

    match state {
        ShortcutState::Pressed => {
            windows::show_overlay(app);
            let event = app_state.engine.on_hotkey_down();
            events::emit_state(app, &event);
        }
        ShortcutState::Released => {
            // Phase 1 will wire real active-app detection; Phase 0 leaves this
            // as None (per-app profiles and the sensitive-app blocklist simply
            // won't trigger until that's in place).
            let active_app_id: Option<String> = None;

            match app_state.engine.on_hotkey_up(active_app_id) {
                Ok(_) => {
                    if let Some(event) = app_state.engine.last_event() {
                        events::emit_state(app, &event);
                    }
                }
                Err(e) => {
                    tracing::warn!("dictation failed: {e}");
                }
            }

            windows::hide_overlay_after(app.clone(), Duration::from_millis(900));
        }
    }
}
