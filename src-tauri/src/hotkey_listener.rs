//! Spawns the platform hotkey backend and forwards events to the dictation engine.

use crate::{events, state::AppState, windows};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use voxflow_config::DictationMode;
use voxflow_platform::{default_hotkey_backend, HotkeyBinding, HotkeyEvent};

static LISTENING: AtomicBool = AtomicBool::new(false);

/// Runs the hotkey loop on a dedicated OS thread rather than the Tauri async
/// runtime. `handle_hotkey_event` reaches into `DictationEngine`, whose sync API
/// is built on `Runtime::block_on`; calling that from a tokio worker panics with
/// "Cannot start a runtime from within a runtime".
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

            // `register` is async, so it needs an executor — but it must be gone
            // before we handle any event, or we are back inside a runtime.
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
    let app_state = app.state::<AppState>();
    let mode = app_state.engine.get_settings().dictation_mode;

    match (mode, event) {
        (DictationMode::PushToTalk, HotkeyEvent::Pressed) => start_dictation(app),
        (DictationMode::PushToTalk, HotkeyEvent::Released) => stop_dictation(app),
        (DictationMode::Toggle, HotkeyEvent::Pressed) => {
            if LISTENING.load(Ordering::SeqCst) {
                stop_dictation(app);
            } else {
                start_dictation(app);
            }
        }
        (DictationMode::Toggle, HotkeyEvent::Released) => {}
        (_, HotkeyEvent::Toggled) => {}
    }
}

fn start_dictation(app: &AppHandle) {
    LISTENING.store(true, Ordering::SeqCst);
    windows::show_overlay(app);
    let app_state = app.state::<AppState>();
    let event = app_state.engine.on_hotkey_down();
    events::emit_state(app, &event);
    spawn_amplitude_poller(app.clone());
}

/// Streams the mic input level to the overlay while listening so the waveform
/// reacts to the user's voice. Runs on its own OS thread (not the tokio
/// runtime) because `current_input_level` blocks on the engine's runtime, and
/// exits as soon as `LISTENING` clears on key release.
fn spawn_amplitude_poller(app: AppHandle) {
    let _ = std::thread::Builder::new()
        .name("voxflow-amplitude-poller".into())
        .spawn(move || {
            while LISTENING.load(Ordering::SeqCst) {
                let level = app.state::<AppState>().engine.current_input_level();
                events::emit_amplitude(&app, level);
                std::thread::sleep(Duration::from_millis(33)); // ~30 fps
            }
            // One final zero so the wave settles flat when recording stops.
            events::emit_amplitude(&app, 0.0);
        });
}

fn stop_dictation(app: &AppHandle) {
    LISTENING.store(false, Ordering::SeqCst);

    // Leave the "listening" UI state the instant the key is released, before the
    // blocking transcribe/rewrite/insert work runs. Otherwise the overlay keeps
    // showing "Listening" with a running timer for the 2-3s of processing, which
    // reads as the app failing to stop recording.
    events::emit_state(
        app,
        &voxflow_core::StateEvent::new(voxflow_core::DictationState::Transcribing, None),
    );

    let app_state = app.state::<AppState>();
    match app_state.engine.on_hotkey_up(None) {
        Ok(_) => {
            if let Some(event) = app_state.engine.last_event() {
                events::emit_state(app, &event);
            }
        }
        Err(e) => tracing::warn!("dictation failed: {e}"),
    }
    windows::hide_overlay_after(app.clone(), Duration::from_millis(900));
}
