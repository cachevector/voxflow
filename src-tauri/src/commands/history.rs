use crate::state::AppState;
use tauri::State;
use voxflow_core::{HistoryCorrectionResult, VocabularySuggestion};
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

#[tauri::command]
pub fn correct_history_entry(
    state: State<'_, AppState>,
    id: String,
    corrected_text: String,
) -> Result<HistoryCorrectionResult, String> {
    state
        .engine
        .correct_history_entry(id, corrected_text)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn accept_vocabulary_suggestion(
    state: State<'_, AppState>,
    suggestion: VocabularySuggestion,
) -> Result<(), String> {
    state
        .engine
        .accept_vocabulary_suggestion(suggestion)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn dismiss_vocabulary_suggestion(
    state: State<'_, AppState>,
    suggestion: VocabularySuggestion,
) -> Result<(), String> {
    state
        .engine
        .dismiss_vocabulary_suggestion(suggestion)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_vocabulary_suggestion(
    state: State<'_, AppState>,
    suggestion: VocabularySuggestion,
) -> Result<(), String> {
    state
        .engine
        .restore_vocabulary_suggestion(suggestion)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn respond_to_edit_learning_suggestion(
    app: tauri::AppHandle,
    accepted: bool,
) -> Result<(), String> {
    crate::edit_learning_shortcuts::respond(&app, accepted);
    Ok(())
}
