mod commands;
mod events;
mod hotkey_listener;
mod state;
mod tray;
mod windows;

use state::AppState;
use std::sync::Arc;
use tauri::Manager;
use voxflow_core::DictationEngine;
use voxflow_insert::{ClipboardPasteInserter, InsertionBridge};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            crate::windows::show_main(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
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
            app.manage(AppState {
                engine: engine.clone(),
            });

            if let Err(e) = engine.prewarm() {
                tracing::warn!("prewarm failed (mic/model may download on first dictation): {e}");
            }

            hotkey_listener::spawn(app.handle().clone());
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
            commands::permissions::get_permission_status,
            commands::permissions::open_accessibility_settings,
            commands::permissions::paste_text,
            commands::permissions::whisper_model_ready,
            commands::permissions::download_whisper_model,
            commands::permissions::complete_onboarding,
        ])
        .run(tauri::generate_context!())
        .expect("error while running voxflow");
}
