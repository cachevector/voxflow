use crate::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize)]
pub struct PermissionStatus {
    pub accessibility_hint: String,
    pub microphone_hint: String,
}

#[tauri::command]
pub fn get_permission_status() -> PermissionStatus {
    PermissionStatus {
        accessibility_hint: "System Settings → Privacy & Security → Accessibility — enable VoxFlow for global Option+Ctrl and paste.".into(),
        microphone_hint: "System Settings → Privacy & Security → Microphone — enable VoxFlow.".into(),
    }
}

#[tauri::command]
pub fn open_accessibility_settings(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
    }
    Ok(())
}

#[tauri::command]
pub fn paste_text(app: AppHandle, text: String) -> Result<voxflow_insert::InsertResult, String> {
    let state = app.state::<AppState>();
    state
        .engine
        .paste_text(text)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn whisper_model_ready(app: AppHandle) -> bool {
    let state = app.state::<AppState>();
    state.engine.whisper_model_ready()
}

#[tauri::command]
pub async fn download_whisper_model(app: AppHandle) -> Result<String, String> {
    // `get_settings`/`prewarm` use the engine's own runtime internally, so they
    // must not run on this async command's tokio worker.
    let settings_app = app.clone();
    let model_id = tauri::async_runtime::spawn_blocking(move || {
        settings_app
            .state::<AppState>()
            .engine
            .get_settings()
            .whisper
            .model_id
    })
    .await
    .map_err(|e| e.to_string())?;

    let path = voxflow_whisper::ensure_model_downloaded(&model_id)
        .await
        .map_err(|e| e.to_string())?;

    tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) = app.state::<AppState>().engine.prewarm() {
            tracing::warn!("prewarm after model download failed: {e}");
        }
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(path.display().to_string())
}

#[tauri::command]
pub fn complete_onboarding(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut settings = state.engine.get_settings();
    settings.onboarding_complete = true;
    state
        .engine
        .save_settings(settings)
        .map_err(|e| e.to_string())
}
