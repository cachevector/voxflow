//! Temporary global shortcuts for the manual-edit vocabulary prompt.
//!
//! They are registered only while a prompt is visible, so VoxFlow never
//! reserves Escape or Option+Return during normal use.

use crate::{events, state::AppState, windows};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use voxflow_core::VocabularySuggestion;

fn learn_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::ALT), Code::Enter)
}

fn dismiss_shortcut() -> Shortcut {
    Shortcut::new(None, Code::Escape)
}

pub fn show(app: &AppHandle, suggestion: VocabularySuggestion) {
    // A second completed dictation can replace a pending suggestion. Remove
    // the old bindings first, then replace the in-memory prompt atomically.
    unregister(app);
    let state = app.state::<AppState>();
    *state.edit_learning_suggestion.lock() = Some(suggestion.clone());
    let revision = state.edit_learning_revision.fetch_add(1, Ordering::SeqCst) + 1;

    let learn = learn_shortcut();
    let dismiss = dismiss_shortcut();
    if let Err(error) =
        app.global_shortcut()
            .on_shortcuts([learn, dismiss], |app, shortcut, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                let accepted = shortcut.matches(Modifiers::ALT, Code::Enter);
                if accepted || shortcut.matches(Modifiers::empty(), Code::Escape) {
                    respond(app, accepted);
                }
            })
    {
        // The click controls remain usable if another app owns either shortcut.
        tracing::debug!("manual-edit response shortcut unavailable: {error}");
    }

    events::emit_vocabulary_suggestion(app, &suggestion);
    let timeout_app = app.clone();
    let _ = std::thread::Builder::new()
        .name("voxflow-edit-learning-timeout".into())
        .spawn(move || {
            std::thread::sleep(Duration::from_secs(15));
            clear_if_current(&timeout_app, revision);
        });
}

pub fn respond(app: &AppHandle, accepted: bool) {
    let suggestion = app
        .state::<AppState>()
        .edit_learning_suggestion
        .lock()
        .take();
    let Some(suggestion) = suggestion else {
        return;
    };

    let result = if accepted {
        app.state::<AppState>()
            .engine
            .accept_vocabulary_suggestion(suggestion)
    } else {
        app.state::<AppState>()
            .engine
            .dismiss_vocabulary_suggestion(suggestion)
    };
    if let Err(error) = result {
        tracing::warn!("failed to store manual-edit vocabulary response: {error}");
    }
    finish(app);
}

pub fn clear(app: &AppHandle) {
    if app
        .state::<AppState>()
        .edit_learning_suggestion
        .lock()
        .take()
        .is_some()
    {
        finish(app);
    }
}

fn clear_if_current(app: &AppHandle, revision: u64) {
    if app
        .state::<AppState>()
        .edit_learning_revision
        .load(Ordering::SeqCst)
        == revision
    {
        clear(app);
    }
}

fn finish(app: &AppHandle) {
    unregister(app);
    events::emit_vocabulary_suggestion_cleared(app);
    windows::hide_overlay_after(app.clone(), Duration::from_millis(150));
}

fn unregister(app: &AppHandle) {
    let _ = app
        .global_shortcut()
        .unregister_multiple([learn_shortcut(), dismiss_shortcut()]);
}
