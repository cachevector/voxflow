use crate::{FfiCostDashboard, FfiDictationResult, FfiSettings, FfiUiState};
use parking_lot::Mutex;
use std::sync::Arc;
use voxflow_config::{load_settings, save_settings, Settings};
use voxflow_core::{apply_license, beta_invite_valid, DictationEngine};
use voxflow_insert::{InsertionBridge, StubInserter};

struct EngineHolder {
    engine: DictationEngine,
}

impl EngineHolder {
    fn new(api_key: Option<String>) -> Result<Self, anyhow::Error> {
        let mut settings = load_settings().unwrap_or_default();
        if let Some(key) = api_key {
            settings.openai_api_key = Some(key);
        }
        let stub = Arc::new(InsertionBridge::new(Box::new(StubInserter::default())));
        let engine = DictationEngine::new(settings, stub)?;
        Ok(Self { engine })
    }
}

#[derive(uniffi::Object)]
pub struct VoxFlowEngine {
    inner: Mutex<EngineHolder>,
}

#[uniffi::export]
impl VoxFlowEngine {
    #[uniffi::constructor]
    pub fn new(api_key: Option<String>) -> Arc<Self> {
        let holder = EngineHolder::new(api_key).expect("failed to init VoxFlow engine");
        Arc::new(Self {
            inner: Mutex::new(holder),
        })
    }

    pub fn prewarm(&self) {
        let _ = self.inner.lock().engine.prewarm();
    }

    pub fn on_hotkey_down(&self) -> FfiUiState {
        let event = self.inner.lock().engine.on_hotkey_down();
        map_ui_state(event.ui_state)
    }

    pub fn on_hotkey_up(&self, active_app_id: Option<String>) -> FfiDictationResult {
        match self.inner.lock().engine.on_hotkey_up(active_app_id) {
            Ok(result) => FfiDictationResult {
                text: result.text,
                ui_state: map_ui_state(result.state.into()),
                latency_ms: result.latency_ms,
                success: matches!(result.state, voxflow_core::DictationState::Done),
            },
            Err(e) => {
                tracing::error!("dictation failed: {e:#}");
                FfiDictationResult {
                    text: String::new(),
                    ui_state: FfiUiState::Error,
                    latency_ms: 0,
                    success: false,
                }
            }
        }
    }

    pub fn get_settings(&self) -> FfiSettings {
        map_settings(&self.inner.lock().engine.get_settings())
    }

    pub fn save_settings(&self, settings: FfiSettings) {
        let mut s = self.inner.lock().engine.get_settings();
        apply_ffi_settings(&mut s, &settings);
        let _ = save_settings(&s);
        let _ = self.inner.lock().engine.save_settings(s);
    }

    pub fn cost_dashboard(&self) -> FfiCostDashboard {
        let d = self.inner.lock().engine.cost_dashboard();
        FfiCostDashboard {
            minutes_transcribed: d.minutes_transcribed,
            estimated_usd: d.estimated_usd,
            estimated_inr: d.estimated_inr,
            projected_monthly_usd: d.projected_monthly_usd,
            projected_monthly_inr: d.projected_monthly_inr,
            local_percentage: d.local_percentage,
            cloud_percentage: d.cloud_percentage,
            cap_warnings: d.cap_warnings,
        }
    }

    pub fn validate_license(&self, key: String) -> String {
        let mut settings = self.inner.lock().engine.get_settings();
        let result = apply_license(&mut settings, &key);
        let _ = self.inner.lock().engine.save_settings(settings);
        result.message
    }

    pub fn beta_invite_valid(&self, code: String) -> bool {
        beta_invite_valid(&code)
    }

    pub fn list_audio_devices(&self) -> Vec<String> {
        voxflow_audio::AudioCapture::list_devices()
            .map(|devices| devices.into_iter().map(|d| d.name).collect())
            .unwrap_or_default()
    }
}

fn map_ui_state(state: voxflow_core::UiState) -> FfiUiState {
    match state {
        voxflow_core::UiState::Idle => FfiUiState::Idle,
        voxflow_core::UiState::Listening => FfiUiState::Listening,
        voxflow_core::UiState::Cleaning => FfiUiState::Cleaning,
        voxflow_core::UiState::Inserting => FfiUiState::Inserting,
        voxflow_core::UiState::Copied => FfiUiState::Copied,
        voxflow_core::UiState::Done => FfiUiState::Done,
        voxflow_core::UiState::Error => FfiUiState::Error,
    }
}

fn map_settings(s: &Settings) -> FfiSettings {
    FfiSettings {
        quality_mode: format!("{:?}", s.quality_mode),
        openai_api_key: s.openai_api_key.clone(),
        openai_model: s.openai_model.clone(),
        microphone_device: s.microphone_device.clone(),
        clipboard_restore: s.clipboard_restore,
        onboarding_complete: s.onboarding_complete,
        is_pro: s.is_pro(),
        hotkey_key_code: s.hotkey.key_code,
        hotkey_modifiers: s.hotkey.modifiers,
        hotkey_label: s.hotkey.label.clone(),
    }
}

fn apply_ffi_settings(s: &mut Settings, f: &FfiSettings) {
    s.openai_api_key = f.openai_api_key.clone();
    s.openai_model = f.openai_model.clone();
    s.microphone_device = f.microphone_device.clone();
    s.clipboard_restore = f.clipboard_restore;
    s.onboarding_complete = f.onboarding_complete;
}
