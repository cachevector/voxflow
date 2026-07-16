use std::sync::Arc;
use voxflow_core::DictationEngine;

pub struct AppState {
    pub engine: Arc<DictationEngine>,
}
