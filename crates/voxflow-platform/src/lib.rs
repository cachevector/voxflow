mod hotkey;
mod manual_binding;
mod session;
#[cfg(target_os = "linux")]
mod portal_hotkey;
#[cfg(target_os = "linux")]
mod x11_hotkey;
#[cfg(target_os = "macos")]
mod macos_modifier_hotkey;

pub use hotkey::{
    AutostartBackend, Code, GlobalHotkeyBackend, HotkeyBinding, HotkeyError, HotkeyEvent,
    Modifiers,
};
pub use manual_binding::ManualBindingHotkey;
pub use session::{detect_session, DesktopSession, WaylandCompositor};
#[cfg(target_os = "linux")]
pub use portal_hotkey::PortalHotkey;
#[cfg(target_os = "linux")]
pub use x11_hotkey::X11Hotkey;
#[cfg(target_os = "macos")]
pub use macos_modifier_hotkey::MacModifierHotkey;

/// Pick the best hotkey backend for the current OS/session.
pub fn default_hotkey_backend() -> Result<Box<dyn GlobalHotkeyBackend>, HotkeyError> {
    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(MacModifierHotkey::new()))
    }
    #[cfg(target_os = "linux")]
    {
        let session = detect_session();
        if let DesktopSession::LinuxWayland { compositor } = session {
            if compositor.supports_global_shortcuts_portal() {
                if let Ok(portal) = PortalHotkey::new() {
                    return Ok(Box::new(portal));
                }
            }
        }
        X11Hotkey::new().map(|h| Box::new(h) as Box<dyn GlobalHotkeyBackend>)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(HotkeyError::Unsupported)
    }
}
