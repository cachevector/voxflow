mod hotkey;
mod manual_binding;
mod session;
#[cfg(target_os = "linux")]
mod x11_hotkey;

pub use hotkey::{
    AutostartBackend, Code, GlobalHotkeyBackend, HotkeyBinding, HotkeyError, HotkeyEvent,
    Modifiers,
};
pub use manual_binding::ManualBindingHotkey;
pub use session::{detect_session, DesktopSession, WaylandCompositor};
#[cfg(target_os = "linux")]
pub use x11_hotkey::X11Hotkey;
