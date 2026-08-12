//! Wayland global shortcuts via the XDG Desktop Portal (`ashpd`).
//!
//! When the portal is unavailable (X11 session, missing portal implementation),
//! [`crate::default_hotkey_backend`] falls back to [`super::X11Hotkey`].

use crate::hotkey::{GlobalHotkeyBackend, HotkeyBinding, HotkeyError, HotkeyEvent};
use async_trait::async_trait;
use tracing::warn;

pub struct PortalHotkey {
    tx: async_channel::Sender<HotkeyEvent>,
    rx: async_channel::Receiver<HotkeyEvent>,
}

impl PortalHotkey {
    pub fn new() -> Result<Self, HotkeyError> {
        let (tx, rx) = async_channel::unbounded();
        Ok(Self { tx, rx })
    }
}

#[async_trait]
impl GlobalHotkeyBackend for PortalHotkey {
    async fn register(&self, binding: HotkeyBinding) -> Result<(), HotkeyError> {
        let tx = self.tx.clone();
        let label = binding.label.clone();
        tokio::spawn(async move {
            if let Err(e) = run_portal_session(tx, &label).await {
                warn!("GlobalShortcuts portal session ended: {e}");
            }
        });
        Ok(())
    }

    async fn unregister(&self) -> Result<(), HotkeyError> {
        Ok(())
    }

    fn events(&self) -> async_channel::Receiver<HotkeyEvent> {
        self.rx.clone()
    }
}

async fn run_portal_session(
    tx: async_channel::Sender<HotkeyEvent>,
    shortcut_label: &str,
) -> Result<(), HotkeyError> {
    use ashpd::desktop::global_shortcuts::{GlobalShortcuts, GlobalShortcutsUpdate};
    use futures_util::StreamExt;

    let proxy = GlobalShortcuts::new()
        .await
        .map_err(|e| HotkeyError::Other(e.into()))?;

    let session = proxy
        .create_session()
        .await
        .map_err(|e| HotkeyError::Other(e.into()))?;

    let id = "voxflow-dictation";
    let shortcuts = vec![(id, shortcut_label, "")];
    session
        .bind_shortcuts(&shortcuts, None::<&str>, None)
        .await
        .map_err(|e| HotkeyError::Other(e.into()))?;

    let mut updates = session
        .receive()
        .await
        .map_err(|e| HotkeyError::Other(e.into()))?;

    while let Some(update) = updates.next().await {
        match update {
            GlobalShortcutsUpdate::Activated { shortcut, .. } if shortcut == id => {
                let _ = tx.send(HotkeyEvent::Pressed).await;
            }
            GlobalShortcutsUpdate::Deactivated { shortcut, .. } if shortcut == id => {
                let _ = tx.send(HotkeyEvent::Released).await;
            }
            _ => {}
        }
    }
    Ok(())
}
