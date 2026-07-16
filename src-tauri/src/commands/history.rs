use crate::state::AppState;
use tauri::State;
use voxflow_history::HistoryEntry;

#[tauri::command]
pub fn list_history(state: State<'_, AppState>, limit: u32) -> Result<Vec<HistoryEntry>, String> {
    state.engine.list_history(limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_history_json(state: State<'_, AppState>, limit: u32) -> Result<String, String> {
    state
        .engine
        .export_history_json(limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_history_csv(state: State<'_, AppState>, limit: u32) -> Result<String, String> {
    state
        .engine
        .export_history_csv(limit)
        .map_err(|e| e.to_string())
}
