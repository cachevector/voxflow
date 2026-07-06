mod voxflow_engine;

pub use voxflow_engine::VoxFlowEngine;

use uniffi::{Enum, Record};

#[derive(Clone, Debug, Enum)]
pub enum FfiUiState {
    Idle,
    Listening,
    Cleaning,
    Inserting,
    Copied,
    Done,
    Error,
}

#[derive(Clone, Debug, Record)]
pub struct FfiDictationResult {
    pub text: String,
    pub ui_state: FfiUiState,
    pub latency_ms: u64,
    pub success: bool,
}

#[derive(Clone, Debug, Record)]
pub struct FfiCostDashboard {
    pub minutes_transcribed: f32,
    pub estimated_usd: f32,
    pub estimated_inr: f32,
    pub projected_monthly_usd: f32,
    pub projected_monthly_inr: f32,
    pub local_percentage: f32,
    pub cloud_percentage: f32,
    pub cap_warnings: Vec<String>,
}

#[derive(Clone, Debug, Record)]
pub struct FfiSettings {
    pub quality_mode: String,
    pub openai_api_key: Option<String>,
    pub openai_model: String,
    pub microphone_device: Option<String>,
    pub clipboard_restore: bool,
    pub onboarding_complete: bool,
    pub is_pro: bool,
    pub hotkey_key_code: u16,
    pub hotkey_modifiers: u32,
    pub hotkey_label: String,
}

uniffi::setup_scaffolding!();
