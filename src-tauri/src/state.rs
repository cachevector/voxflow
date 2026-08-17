use parking_lot::Mutex;
use std::sync::{atomic::AtomicU64, Arc};
use voxflow_core::DictationEngine;

pub struct AppState {
    pub engine: Arc<DictationEngine>,
    /// Present only while the short-lived manual-edit prompt is visible.
    pub edit_learning_suggestion: Mutex<Option<voxflow_core::VocabularySuggestion>>,
    pub edit_learning_revision: AtomicU64,
}
