use crate::state::AppState;
use tauri::State;
use voxflow_cost::CostDashboard;

#[tauri::command]
pub fn get_cost_dashboard(state: State<'_, AppState>) -> CostDashboard {
    state.engine.cost_dashboard()
}
