use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DictationState {
    Idle,
    Listening,
    Finalizing,
    Transcribing,
    Cleaning,
    Inserting,
    Copied,
    Done,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiState {
    Idle,
    Listening,
    Transcribing,
    Cleaning,
    Inserting,
    Copied,
    Done,
    Error,
}

impl From<DictationState> for UiState {
    fn from(state: DictationState) -> Self {
        match state {
            DictationState::Idle => UiState::Idle,
            DictationState::Listening => UiState::Listening,
            DictationState::Finalizing | DictationState::Transcribing => UiState::Transcribing,
            DictationState::Cleaning => UiState::Cleaning,
            DictationState::Inserting => UiState::Inserting,
            DictationState::Copied => UiState::Copied,
            DictationState::Done => UiState::Done,
            DictationState::Error => UiState::Error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEvent {
    #[serde(default)]
    pub session_id: u64,
    pub state: DictationState,
    pub ui_state: UiState,
    pub message: Option<String>,
    pub transcript_preview: Option<String>,
}

impl StateEvent {
    pub fn new(state: DictationState, message: Option<String>) -> Self {
        Self {
            session_id: 0,
            ui_state: state.into(),
            state,
            message,
            transcript_preview: None,
        }
    }
}
