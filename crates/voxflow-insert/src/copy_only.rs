use crate::{InsertError, InsertMethod, InsertResult, TextInserter};
use arboard::Clipboard;
use async_trait::async_trait;

/// GNOME Wayland's default insertion path (per the Phase 0.7 spike
/// deferral): copies the cleaned-up text to the clipboard and stops there —
/// no synthetic paste is attempted. GNOME Wayland has no working
/// XTest-equivalent, and the `RemoteDesktop` portal's synthetic-paste
/// reliability is unverified, so this is the honest, always-works fallback
/// rather than a degraded error state. The overlay surfaces a
/// "Copied — press Ctrl+V" state so the user knows to paste manually.
pub struct CopyOnlyInserter;

#[async_trait]
impl TextInserter for CopyOnlyInserter {
    async fn insert(
        &self,
        text: &str,
        _restore_clipboard: bool,
    ) -> Result<InsertResult, InsertError> {
        if text.trim().is_empty() {
            return Err(InsertError::EmptyText);
        }
        self.copy_only(text).await?;
        Ok(InsertResult {
            success: true,
            method: InsertMethod::CopyOnly,
            restored_clipboard: false,
        })
    }

    async fn copy_only(&self, text: &str) -> Result<(), InsertError> {
        let mut clipboard =
            Clipboard::new().map_err(|e| InsertError::PasteFailed(e.to_string()))?;
        clipboard
            .set_text(text.to_string())
            .map_err(|e| InsertError::PasteFailed(e.to_string()))
    }
}
