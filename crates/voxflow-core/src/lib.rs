pub mod engine;
pub mod events;
pub mod latency;
pub mod license;
pub mod pipeline;

pub use engine::DictationEngine;
pub use events::{DictationState, StateEvent, UiState};
pub use latency::LatencyTracker;
pub use license::{apply_license, beta_invite_valid, validate_license_key, LicenseValidation};
pub use pipeline::{DictationPipeline, HistoryCorrectionResult, VocabularySuggestion};
