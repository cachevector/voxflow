use async_trait::async_trait;
use thiserror::Error;

// Re-exported so callers can build a `HotkeyBinding` without depending on
// `global-hotkey` directly. `ManualBindingHotkey` ignores the binding's
// contents entirely (the actual key combo lives in the user's compositor
// config), but `X11Hotkey`/`MacHotkey` need real modifier/code values.
pub use global_hotkey::hotkey::{Code, Modifiers};

/// A global push-to-talk (or toggle) key binding, independent of how the
/// concrete backend actually grabs it (XGrabKey, a portal, or a manual
/// compositor-bound trigger over a socket).
#[derive(Debug, Clone)]
pub struct HotkeyBinding {
    pub modifiers: Modifiers,
    pub code: Code,
    pub label: String,
}

impl HotkeyBinding {
    /// VoxFlow's documented default: Super+Shift+Space. Chosen (in the
    /// original Tauri implementation) specifically as a modifier+key combo
    /// rather than a bare modifier, so no Input Monitoring permission is
    /// needed on macOS for the common case.
    pub fn default_binding() -> Self {
        Self {
            modifiers: Modifiers::ALT | Modifiers::CONTROL,
            code: Code::Space,
            label: "Option+Ctrl".into(),
        }
    }

    /// Legacy combo kept for X11/portal backends that require a real key.
    pub fn legacy_combo_binding() -> Self {
        Self {
            modifiers: Modifiers::SUPER | Modifiers::SHIFT,
            code: Code::Space,
            label: "Super+Shift+Space".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    Pressed,
    Released,
    /// Emitted by backends that can only observe a single trigger per press
    /// (e.g. `ManualBindingHotkey` on GNOME) — the caller is responsible for
    /// turning this into start/stop semantics based on current state.
    Toggled,
}

#[derive(Debug, Error)]
pub enum HotkeyError {
    #[error("no supported hotkey backend for this session; use `voxflowctl trigger` bound to a manual compositor keybinding instead")]
    Unsupported,
    #[error("hotkey already bound by another application: {0}")]
    Conflict(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Implemented by each platform/session-specific hotkey mechanism
/// (`X11Hotkey`, `PortalHotkey`, `ManualBindingHotkey`, `MacHotkey`).
#[async_trait]
pub trait GlobalHotkeyBackend: Send + Sync {
    async fn register(&self, binding: HotkeyBinding) -> Result<(), HotkeyError>;
    async fn unregister(&self) -> Result<(), HotkeyError>;
    /// The backend pushes press/release/toggle events here; the caller owns
    /// the receiving end and decides how to map them onto the dictation
    /// engine's start/stop calls.
    fn events(&self) -> async_channel::Receiver<HotkeyEvent>;
}

/// Launch-at-login management. Implementations write/remove an XDG
/// `.desktop` autostart entry on Linux or a launchd agent on macOS.
pub trait AutostartBackend: Send + Sync {
    fn is_enabled(&self) -> bool;
    fn set_enabled(&self, enabled: bool) -> anyhow::Result<()>;
}
