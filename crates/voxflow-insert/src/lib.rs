use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod clipboard_paste;
pub use clipboard_paste::ClipboardPasteInserter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsertMethod {
    Accessibility,
    ClipboardPaste,
    CopyOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertResult {
    pub success: bool,
    pub method: InsertMethod,
    pub restored_clipboard: bool,
}

#[derive(Debug, Error)]
pub enum InsertError {
    #[error("insertion blocked for app: {0}")]
    BlockedApp(String),
    #[error("paste failed: {0}")]
    PasteFailed(String),
    #[error("no text to insert")]
    EmptyText,
}

#[async_trait]
pub trait TextInserter: Send + Sync {
    async fn insert(
        &self,
        text: &str,
        restore_clipboard: bool,
    ) -> Result<InsertResult, InsertError>;
    async fn copy_only(&self, text: &str) -> Result<(), InsertError>;
}

/// Platform shells implement TextInserter; Rust core uses this callback holder.
pub struct InsertionBridge {
    inner: Box<dyn TextInserter>,
}

impl InsertionBridge {
    pub fn new(inner: Box<dyn TextInserter>) -> Self {
        Self { inner }
    }

    pub async fn insert(
        &self,
        text: &str,
        restore_clipboard: bool,
    ) -> Result<InsertResult, InsertError> {
        if text.trim().is_empty() {
            return Err(InsertError::EmptyText);
        }
        self.inner.insert(text, restore_clipboard).await
    }

    pub async fn copy_only(&self, text: &str) -> Result<(), InsertError> {
        self.inner.copy_only(text).await
    }
}

/// Stub inserter for tests and headless runs.
pub struct StubInserter {
    pub last_text: parking_lot::Mutex<Option<String>>,
}

impl Default for StubInserter {
    fn default() -> Self {
        Self {
            last_text: parking_lot::Mutex::new(None),
        }
    }
}

#[async_trait]
impl TextInserter for StubInserter {
    async fn insert(
        &self,
        text: &str,
        _restore_clipboard: bool,
    ) -> Result<InsertResult, InsertError> {
        *self.last_text.lock() = Some(text.to_string());
        Ok(InsertResult {
            success: true,
            method: InsertMethod::Accessibility,
            restored_clipboard: false,
        })
    }

    async fn copy_only(&self, text: &str) -> Result<(), InsertError> {
        *self.last_text.lock() = Some(text.to_string());
        Ok(())
    }
}
