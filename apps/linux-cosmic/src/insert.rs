use async_trait::async_trait;
use std::process::Command;

use voxflow_insert::{InsertError, InsertMethod, InsertResult, TextInserter};

pub struct LinuxClipboardInserter;

#[async_trait]
impl TextInserter for LinuxClipboardInserter {
    async fn insert(&self, text: &str, restore_clipboard: bool) -> Result<InsertResult, InsertError> {
        if text.trim().is_empty() {
            return Err(InsertError::EmptyText);
        }

        let previous = arboard::Clipboard::new()
            .ok()
            .and_then(|mut cb| cb.get_text().ok());

        arboard::Clipboard::new()
            .map_err(|e| InsertError::PasteFailed(e.to_string()))?
            .set_text(text.to_string())
            .map_err(|e| InsertError::PasteFailed(e.to_string()))?;

        // Simulate Ctrl+V via xdotool if available (X11); Wayland may require manual paste
        let paste_ok = Command::new("xdotool")
            .args(["key", "ctrl+v"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if restore_clipboard {
            if let Some(prev) = previous {
                let _ = arboard::Clipboard::new().and_then(|mut cb| cb.set_text(prev));
            }
        }

        Ok(InsertResult {
            success: paste_ok,
            method: if paste_ok {
                InsertMethod::ClipboardPaste
            } else {
                InsertMethod::CopyOnly
            },
            restored_clipboard: restore_clipboard,
        })
    }

    async fn copy_only(&self, text: &str) -> Result<(), InsertError> {
        arboard::Clipboard::new()
            .map_err(|e| InsertError::PasteFailed(e.to_string()))?
            .set_text(text.to_string())
            .map_err(|e| InsertError::PasteFailed(e.to_string()))?;
        Ok(())
    }
}
