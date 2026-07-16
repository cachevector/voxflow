use crate::{InsertError, InsertMethod, InsertResult, TextInserter};
use arboard::Clipboard;
use async_trait::async_trait;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::time::Duration;

/// Cross-platform clipboard + simulated-paste inserter. This is the primary
/// reliable insertion path on both macOS and Windows for MVP — direct
/// Accessibility (macOS) / UI Automation (Windows) insertion is a Phase 1
/// enhancement layered in front of this fallback, not a replacement for it.
pub struct ClipboardPasteInserter {
    restore_delay: Duration,
}

impl Default for ClipboardPasteInserter {
    fn default() -> Self {
        Self {
            restore_delay: Duration::from_millis(500),
        }
    }
}

#[async_trait]
impl TextInserter for ClipboardPasteInserter {
    async fn insert(
        &self,
        text: &str,
        restore_clipboard: bool,
    ) -> Result<InsertResult, InsertError> {
        if text.trim().is_empty() {
            return Err(InsertError::EmptyText);
        }

        let mut clipboard =
            Clipboard::new().map_err(|e| InsertError::PasteFailed(e.to_string()))?;
        let previous = if restore_clipboard {
            clipboard.get_text().ok()
        } else {
            None
        };

        clipboard
            .set_text(text.to_string())
            .map_err(|e| InsertError::PasteFailed(e.to_string()))?;

        simulate_paste().map_err(|e| InsertError::PasteFailed(e.to_string()))?;

        let restored_clipboard = if let Some(prev) = previous {
            tokio::time::sleep(self.restore_delay).await;
            clipboard.set_text(prev).is_ok()
        } else {
            false
        };

        Ok(InsertResult {
            success: true,
            method: InsertMethod::ClipboardPaste,
            restored_clipboard,
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

fn simulate_paste() -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    let modifier = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;

    enigo
        .key(modifier, Direction::Press)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| e.to_string())?;
    enigo
        .key(modifier, Direction::Release)
        .map_err(|e| e.to_string())?;
    Ok(())
}
