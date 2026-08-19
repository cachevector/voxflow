//! Spawns the platform hotkey backend and coordinates one dictation session at a time.

use crate::{events, state::AppState, windows};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use voxflow_config::DictationMode;
use voxflow_core::{DictationState, StateEvent};
use voxflow_platform::{default_hotkey_backend, HotkeyBinding, HotkeyEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Idle,
    Listening {
        session_id: u64,
        mode: DictationMode,
    },
    Processing {
        session_id: u64,
    },
}

#[derive(Debug)]
struct Controller {
    lifecycle: Lifecycle,
    next_session_id: u64,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            lifecycle: Lifecycle::Idle,
            next_session_id: 1,
        }
    }
}

static CONTROLLER: LazyLock<Mutex<Controller>> =
    LazyLock::new(|| Mutex::new(Controller::default()));

fn lifecycle() -> Lifecycle {
    CONTROLLER
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .lifecycle
}

/// Runs the platform event receiver on a dedicated OS thread. Expensive
/// transcription work is dispatched to a separate worker so this loop never
/// accumulates stale press/release pairs while VoxFlow is processing audio.
pub fn spawn(app: AppHandle) {
    let spawned = std::thread::Builder::new()
        .name("voxflow-hotkey-listener".into())
        .spawn(move || {
            let backend = match default_hotkey_backend() {
                Ok(b) => b,
                Err(e) => {
                    tracing::error!("hotkey backend unavailable: {e}");
                    return;
                }
            };

            let registered = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt.block_on(backend.register(HotkeyBinding::default_binding())),
                Err(e) => {
                    tracing::error!("hotkey runtime unavailable: {e}");
                    return;
                }
            };
            if let Err(e) = registered {
                tracing::error!("hotkey registration failed: {e}");
                return;
            }

            let rx = backend.events();
            while let Ok(event) = rx.recv_blocking() {
                handle_hotkey_event(&app, event);
            }
        });

    if let Err(e) = spawned {
        tracing::error!("failed to spawn hotkey listener thread: {e}");
    }
}

fn handle_hotkey_event(app: &AppHandle, event: HotkeyEvent) {
    match lifecycle() {
        Lifecycle::Processing { session_id } => {
            if matches!(event, HotkeyEvent::Pressed | HotkeyEvent::Toggled) {
                tracing::debug!(session_id, "ignored hotkey while dictation is processing");
                emit_session_state(
                    app,
                    StateEvent::new(
                        DictationState::Transcribing,
                        Some("Still processing…".into()),
                    ),
                    session_id,
                );
            }
        }
        Lifecycle::Listening { session_id, mode } => {
            let should_stop = event == HotkeyEvent::Toggled
                || match mode {
                    DictationMode::PushToTalk => event == HotkeyEvent::Released,
                    DictationMode::Toggle => event == HotkeyEvent::Pressed,
                };
            if should_stop {
                stop_dictation(app, session_id);
            }
        }
        Lifecycle::Idle => {
            if !matches!(event, HotkeyEvent::Pressed | HotkeyEvent::Toggled) {
                return;
            }
            let mode = app.state::<AppState>().engine.get_settings().dictation_mode;
            start_dictation(app, mode);
        }
    }
}

fn start_dictation(app: &AppHandle, mode: DictationMode) {
    let session_id = {
        let mut controller = CONTROLLER.lock().unwrap_or_else(|e| e.into_inner());
        if controller.lifecycle != Lifecycle::Idle {
            return;
        }
        let id = controller.next_session_id;
        controller.next_session_id = controller.next_session_id.wrapping_add(1).max(1);
        controller.lifecycle = Lifecycle::Listening {
            session_id: id,
            mode,
        };
        id
    };

    // A pending vocabulary prompt shares the overlay. Clear it before the
    // dictation event so it cannot visually mask the waveform and timer.
    crate::edit_learning_shortcuts::clear(app);

    // Capture starts before window work, and React receives the new state
    // before show(), so stale processing UI is never the first visible frame.
    let event = app.state::<AppState>().engine.on_hotkey_down();
    let start_failed = event.state == DictationState::Error;
    emit_session_state(app, event, session_id);
    windows::show_overlay(app);

    if start_failed {
        set_idle_if_session(session_id);
        windows::hide_overlay_after(app.clone(), Duration::from_millis(1500));
    } else {
        spawn_amplitude_poller(app.clone(), session_id);
    }
}

fn spawn_amplitude_poller(app: AppHandle, session_id: u64) {
    let _ = std::thread::Builder::new()
        .name(format!("voxflow-amplitude-{session_id}"))
        .spawn(move || {
            while matches!(
                lifecycle(),
                Lifecycle::Listening {
                    session_id: active,
                    ..
                } if active == session_id
            ) {
                let level = app.state::<AppState>().engine.current_input_level();
                events::emit_amplitude(&app, level);
                std::thread::sleep(Duration::from_millis(33));
            }
            if !matches!(lifecycle(), Lifecycle::Listening { .. }) {
                events::emit_amplitude(&app, 0.0);
            }
        });
}

fn stop_dictation(app: &AppHandle, session_id: u64) {
    {
        let mut controller = CONTROLLER.lock().unwrap_or_else(|e| e.into_inner());
        if !matches!(
            controller.lifecycle,
            Lifecycle::Listening {
                session_id: active,
                ..
            } if active == session_id
        ) {
            return;
        }
        controller.lifecycle = Lifecycle::Processing { session_id };
    }

    emit_session_state(
        app,
        StateEvent::new(DictationState::Transcribing, None),
        session_id,
    );

    let worker_app = app.clone();
    let spawned = std::thread::Builder::new()
        .name(format!("voxflow-processing-{session_id}"))
        .spawn(move || process_dictation(worker_app, session_id));
    if let Err(error) = spawned {
        tracing::error!(session_id, "failed to spawn dictation worker: {error}");
        emit_session_state(
            app,
            StateEvent::new(
                DictationState::Error,
                Some("Could not start transcription".into()),
            ),
            session_id,
        );
        set_idle_if_session(session_id);
        windows::hide_overlay_after(app.clone(), Duration::from_millis(1500));
    }
}

fn process_dictation(app: AppHandle, session_id: u64) {
    let app_state = app.state::<AppState>();
    let mut hide_delay = Duration::from_millis(150);

    match app_state.engine.on_hotkey_up(None) {
        Ok(result) => {
            let privacy = app_state.engine.get_settings().privacy;
            if privacy.learn_from_manual_edits
                && result
                    .insert_result
                    .as_ref()
                    .is_some_and(|insert| insert.success)
            {
                crate::edit_learning::begin(
                    app.clone(),
                    result.text,
                    privacy.sensitive_app_blocklist,
                );
            }

            if let Some(event) = app_state.engine.last_event() {
                if event.state != DictationState::Done {
                    if matches!(event.state, DictationState::Error | DictationState::Copied) {
                        hide_delay = Duration::from_millis(1500);
                    }
                    emit_session_state(&app, event, session_id);
                }
            }
        }
        Err(error) => {
            tracing::warn!(session_id, "dictation failed: {error}");
            hide_delay = Duration::from_millis(1500);
            emit_session_state(
                &app,
                StateEvent::new(
                    DictationState::Error,
                    Some(format!("Transcription failed: {error}")),
                ),
                session_id,
            );
        }
    }

    set_idle_if_session(session_id);
    windows::hide_overlay_after(app, hide_delay);
}

fn emit_session_state(app: &AppHandle, mut event: StateEvent, session_id: u64) {
    event.session_id = session_id;
    events::emit_state(app, &event);
}

fn set_idle_if_session(session_id: u64) {
    let mut controller = CONTROLLER.lock().unwrap_or_else(|e| e.into_inner());
    let same_session = match controller.lifecycle {
        Lifecycle::Listening {
            session_id: active, ..
        }
        | Lifecycle::Processing { session_id: active } => active == session_id,
        Lifecycle::Idle => false,
    };
    if same_session {
        controller.lifecycle = Lifecycle::Idle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controller_starts_idle_with_nonzero_session_id() {
        let controller = Controller::default();
        assert_eq!(controller.lifecycle, Lifecycle::Idle);
        assert_eq!(controller.next_session_id, 1);
    }

    #[test]
    fn processing_is_distinct_from_listening() {
        assert_ne!(
            Lifecycle::Listening {
                session_id: 7,
                mode: DictationMode::PushToTalk,
            },
            Lifecycle::Processing { session_id: 7 }
        );
    }

    #[test]
    fn manual_toggled_events_can_stop_push_to_talk_sessions() {
        let mode = DictationMode::PushToTalk;
        let event = HotkeyEvent::Toggled;
        let should_stop = event == HotkeyEvent::Toggled
            || match mode {
                DictationMode::PushToTalk => event == HotkeyEvent::Released,
                DictationMode::Toggle => event == HotkeyEvent::Pressed,
            };
        assert!(should_stop);
    }
}
