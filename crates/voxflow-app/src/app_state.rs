use anyhow::{Context, Result};
use std::sync::Arc;
use voxflow_core::DictationEngine;
use voxflow_insert::{ClipboardPasteInserter, CopyOnlyInserter, InsertionBridge};
use voxflow_platform::{DesktopSession, GlobalHotkeyBackend, ManualBindingHotkey, X11Hotkey};

/// Everything voxflow-app wires together at startup: the dictation engine
/// (headless, reused from the old Tauri shell unchanged) and the
/// session-appropriate hotkey backend. Text-insertion backend selection
/// happens once, at `InsertionBridge` construction, and isn't exposed here
/// since `DictationEngine` already owns it internally.
pub struct AppState {
    pub session: DesktopSession,
    pub engine: Arc<DictationEngine>,
    pub hotkey: Arc<dyn GlobalHotkeyBackend>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let session = voxflow_platform::detect_session();
        let settings = voxflow_config::load_settings().context("loading settings")?;

        let inserter = build_inserter(&session);
        let engine = DictationEngine::new(settings, inserter).context("constructing engine")?;

        let hotkey = build_hotkey_backend(&session)?;

        Ok(Self {
            session,
            engine: Arc::new(engine),
            hotkey,
        })
    }
}

/// GNOME Wayland gets `CopyOnlyInserter` per the Phase 0.7 spike deferral
/// (portal-synthesized paste unverified); X11 and macOS get the
/// enigo/XTest-based `ClipboardPasteInserter`, confirmed working in Phase
/// 0.8. Other/unknown Wayland compositors are treated the same as GNOME —
/// conservative default, not an assumption of capability.
fn build_inserter(session: &DesktopSession) -> Arc<InsertionBridge> {
    let inserter: Arc<InsertionBridge> = match session {
        DesktopSession::LinuxX11 | DesktopSession::MacOs => {
            Arc::new(InsertionBridge::new(Box::new(ClipboardPasteInserter::default())))
        }
        DesktopSession::LinuxWayland { .. } => {
            Arc::new(InsertionBridge::new(Box::new(CopyOnlyInserter)))
        }
    };
    inserter
}

/// X11 gets true push-to-talk via `XGrabKey` (Phase 0.6). GNOME Wayland (and
/// any other Wayland compositor without a working `GlobalShortcuts` portal)
/// gets the manual Unix-socket fallback (Phase 0.5), which only supports
/// toggle semantics since GNOME Custom Shortcuts can't observe key-release.
fn build_hotkey_backend(session: &DesktopSession) -> Result<Arc<dyn GlobalHotkeyBackend>> {
    match session {
        DesktopSession::LinuxX11 => {
            Ok(Arc::new(X11Hotkey::new().context("constructing X11Hotkey")?))
        }
        DesktopSession::LinuxWayland { .. } => Ok(Arc::new(ManualBindingHotkey::new())),
        DesktopSession::MacOs => {
            anyhow::bail!("macOS hotkey backend not implemented yet (Phase 4)")
        }
    }
}

/// Whether `session`'s hotkey backend only delivers `HotkeyEvent::Toggled`
/// (single-shot) rather than true press/release pairs. Determines whether
/// `hotkey_glue` should use push-to-talk or toggle semantics.
pub fn uses_toggle_mode(session: &DesktopSession) -> bool {
    matches!(session, DesktopSession::LinuxWayland { .. })
}
