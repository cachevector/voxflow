/// The desktop session VoxFlow is running under. Determines which hotkey,
/// text-insertion, and overlay-window backends get selected at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopSession {
    LinuxX11,
    LinuxWayland { compositor: WaylandCompositor },
    MacOs,
}

/// The Wayland compositor family, insofar as it changes which portals/protocols
/// are available. Detected via `XDG_CURRENT_DESKTOP`; unknown compositors are
/// treated the same as `Gnome` (the most conservative/least-capable case) so
/// unrecognized environments fail gracefully rather than assuming capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaylandCompositor {
    Gnome,
    Kde,
    Hyprland,
    GenericWlroots,
    Unknown,
}

impl WaylandCompositor {
    /// `wlr-layer-shell` support: everything except GNOME (Mutter has a
    /// deliberate policy of not implementing this protocol). `Unknown` is
    /// treated as unsupported — no evidence it works, don't assume it does.
    pub fn supports_layer_shell(self) -> bool {
        matches!(self, Self::Kde | Self::Hyprland | Self::GenericWlroots)
    }

    /// `org.freedesktop.portal.GlobalShortcuts`: only KDE (native) and
    /// Hyprland (its own portal fork) implement this today. Plain
    /// `xdg-desktop-portal-wlr` (Sway et al.) and GNOME do not.
    pub fn supports_global_shortcuts_portal(self) -> bool {
        matches!(self, Self::Kde | Self::Hyprland)
    }

    fn from_xdg_current_desktop(value: &str) -> Self {
        let lower = value.to_ascii_lowercase();
        if lower.contains("gnome") {
            Self::Gnome
        } else if lower.contains("kde") {
            Self::Kde
        } else if lower.contains("hyprland") {
            Self::Hyprland
        } else if lower.contains("sway") || lower.contains("wlroots") {
            Self::GenericWlroots
        } else {
            Self::Unknown
        }
    }
}

/// Reads `XDG_SESSION_TYPE` / `WAYLAND_DISPLAY` / `XDG_CURRENT_DESKTOP` to
/// determine the running desktop session. On non-Linux platforms this always
/// returns `MacOs` (the only other target).
pub fn detect_session() -> DesktopSession {
    #[cfg(target_os = "macos")]
    {
        return DesktopSession::MacOs;
    }

    #[cfg(target_os = "linux")]
    {
        detect_linux_session(
            std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
            std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
            std::env::var("XDG_CURRENT_DESKTOP").ok().as_deref(),
        )
    }
}

#[cfg(target_os = "linux")]
fn detect_linux_session(
    session_type: Option<&str>,
    wayland_display: Option<&str>,
    current_desktop: Option<&str>,
) -> DesktopSession {
    let is_wayland = session_type == Some("wayland")
        || wayland_display.is_some_and(|v| !v.is_empty());

    if is_wayland {
        let compositor = current_desktop
            .map(WaylandCompositor::from_xdg_current_desktop)
            .unwrap_or(WaylandCompositor::Unknown);
        DesktopSession::LinuxWayland { compositor }
    } else {
        DesktopSession::LinuxX11
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn plain_x11_session() {
        let s = detect_linux_session(Some("x11"), None, Some("X-Cinnamon"));
        assert_eq!(s, DesktopSession::LinuxX11);
    }

    #[test]
    fn no_session_type_but_wayland_display_set_is_wayland() {
        let s = detect_linux_session(None, Some("wayland-0"), Some("GNOME"));
        assert_eq!(
            s,
            DesktopSession::LinuxWayland {
                compositor: WaylandCompositor::Gnome
            }
        );
    }

    #[test]
    fn gnome_wayland() {
        let s = detect_linux_session(Some("wayland"), Some("wayland-0"), Some("GNOME"));
        assert_eq!(
            s,
            DesktopSession::LinuxWayland {
                compositor: WaylandCompositor::Gnome
            }
        );
        if let DesktopSession::LinuxWayland { compositor } = s {
            assert!(!compositor.supports_layer_shell());
            assert!(!compositor.supports_global_shortcuts_portal());
        }
    }

    #[test]
    fn kde_wayland() {
        let s = detect_linux_session(Some("wayland"), Some("wayland-1"), Some("KDE"));
        if let DesktopSession::LinuxWayland { compositor } = s {
            assert!(compositor.supports_layer_shell());
            assert!(compositor.supports_global_shortcuts_portal());
        } else {
            panic!("expected wayland session");
        }
    }

    #[test]
    fn sway_wayland_has_layer_shell_but_no_portal_hotkeys() {
        let s = detect_linux_session(Some("wayland"), Some("wayland-1"), Some("sway"));
        if let DesktopSession::LinuxWayland { compositor } = s {
            assert!(compositor.supports_layer_shell());
            assert!(!compositor.supports_global_shortcuts_portal());
        } else {
            panic!("expected wayland session");
        }
    }

    #[test]
    fn empty_wayland_display_is_not_wayland() {
        // Forcing X11 for testing (`WAYLAND_DISPLAY=`) must not be misread as Wayland.
        let s = detect_linux_session(None, Some(""), Some("GNOME"));
        assert_eq!(s, DesktopSession::LinuxX11);
    }

    #[test]
    fn unknown_desktop_is_treated_conservatively() {
        let s = detect_linux_session(Some("wayland"), Some("wayland-0"), Some("SomeNewDE"));
        if let DesktopSession::LinuxWayland { compositor } = s {
            assert_eq!(compositor, WaylandCompositor::Unknown);
            assert!(!compositor.supports_layer_shell());
            assert!(!compositor.supports_global_shortcuts_portal());
        } else {
            panic!("expected wayland session");
        }
    }
}
