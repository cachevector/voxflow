mod commands;
mod events;
mod hotkey;
mod state;
mod tray;
mod windows;

use state::AppState;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use voxflow_core::DictationEngine;
use voxflow_insert::{ClipboardPasteInserter, InsertionBridge};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    hotkey::handle_shortcut(app, event.state());
                })
                .build(),
        )
        .setup(|app| {
            let settings = voxflow_config::load_settings().unwrap_or_default();
            if let Some(legacy_key) = voxflow_config::legacy_plaintext_openai_key() {
                if let Err(e) = voxflow_secrets::set_secret("transcription", &legacy_key) {
                    tracing::warn!("failed to migrate legacy API key into keychain: {e}");
                }
            }

            let inserter = Arc::new(InsertionBridge::new(Box::new(
                ClipboardPasteInserter::default(),
            )));
            let engine = Arc::new(
                DictationEngine::new(settings, inserter).expect("failed to start dictation engine"),
            );
            app.manage(AppState { engine });

            // A hotkey collision with another already-running app is a real,
            // recoverable condition (not a bug in VoxFlow) — surface it as a
            // warning rather than aborting startup. Phase 1 will surface this
            // in the UI so the user can rebind instead of only checking logs.
            if let Err(e) = app.global_shortcut().register(hotkey::default_shortcut()) {
                tracing::warn!(
                    "failed to register default hotkey (likely already bound by another app): {e}"
                );
            }
            tray::build(app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::secrets::set_provider_key,
            commands::secrets::delete_provider_key,
            commands::cost::get_cost_dashboard,
            commands::history::list_history,
            commands::history::export_history_json,
            commands::history::export_history_csv,
            commands::audio::list_audio_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running voxflow");
}
