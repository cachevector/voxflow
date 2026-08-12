use crate::events::{DictationState, StateEvent};
use crate::pipeline::{DictationPipeline, DictationResult};
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use voxflow_config::Settings;
use voxflow_insert::InsertionBridge;
use voxflow_whisper::WhisperEngine;

pub struct DictationEngine {
    runtime: Runtime,
    pipeline: Arc<RwLock<DictationPipeline>>,
    last_event: Mutex<Option<StateEvent>>,
}

impl DictationEngine {
    pub fn new(settings: Settings, inserter: Arc<InsertionBridge>) -> Result<Self> {
        let runtime = Runtime::new()?;
        let whisper = Arc::new(WhisperEngine::new(settings.whisper.model_id.clone()));
        let pipeline = DictationPipeline::new(settings, inserter, whisper)?;
        Ok(Self {
            runtime,
            pipeline: Arc::new(RwLock::new(pipeline)),
            last_event: Mutex::new(None),
        })
    }

    pub fn block_on<F, T>(&self, f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        self.runtime.block_on(f)
    }

    pub fn prewarm(&self) -> Result<()> {
        self.block_on(async { self.pipeline.write().await.prewarm().await })
    }

    pub fn on_hotkey_down(&self) -> StateEvent {
        self.block_on(async {
            let mut p = self.pipeline.write().await;
            let event = p.start_listening().await;
            *self.last_event.lock() = Some(event.clone());
            event
        })
    }

    pub fn on_hotkey_up(&self, active_app_id: Option<String>) -> Result<DictationResult> {
        self.block_on(async {
            let mut p = self.pipeline.write().await;
            let (event, result) = p.stop_and_process(active_app_id).await?;
            *self.last_event.lock() = Some(event);
            Ok(result)
        })
    }

    pub fn current_state(&self) -> DictationState {
        self.block_on(async { self.pipeline.read().await.state() })
    }

    pub fn last_event(&self) -> Option<StateEvent> {
        self.last_event.lock().clone()
    }

    pub fn get_settings(&self) -> Settings {
        self.block_on(async { self.pipeline.read().await.settings().await })
    }

    pub fn save_settings(&self, settings: Settings) -> Result<()> {
        self.block_on(async { self.pipeline.read().await.update_settings(settings).await })
    }

    pub fn cost_dashboard(&self) -> voxflow_cost::CostDashboard {
        self.block_on(async {
            let p = self.pipeline.read().await;
            let settings = p.settings().await;
            p.cost_dashboard(&settings)
        })
    }

    pub fn list_history(&self, limit: u32) -> Result<Vec<voxflow_history::HistoryEntry>> {
        self.block_on(async { self.pipeline.read().await.list_history(limit) })
    }

    pub fn export_history_json(&self, limit: u32) -> Result<String> {
        self.block_on(async { self.pipeline.read().await.export_history_json(limit) })
    }

    pub fn export_history_csv(&self, limit: u32) -> Result<String> {
        self.block_on(async { self.pipeline.read().await.export_history_csv(limit) })
    }

    pub fn paste_text(&self, text: String) -> Result<voxflow_insert::InsertResult> {
        self.block_on(async { self.pipeline.read().await.paste_text(&text).await })
    }

    pub fn whisper_model_ready(&self) -> bool {
        let id = self.block_on(async { self.pipeline.read().await.settings().await.whisper.model_id });
        voxflow_whisper::model_exists(&id)
    }
}
