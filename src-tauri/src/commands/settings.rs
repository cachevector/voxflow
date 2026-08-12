use crate::state::AppState;
use tauri::State;
use voxflow_config::Settings;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.engine.get_settings()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    state
        .engine
        .save_settings(settings)
        .map_err(|e| e.to_string())
}
