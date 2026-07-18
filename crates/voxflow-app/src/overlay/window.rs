use gpui::{
    point, px, Bounds, DisplayId, Pixels, Size, WindowBackgroundAppearance, WindowBounds,
    WindowDecorations, WindowKind, WindowOptions,
};
use voxflow_platform::DesktopSession;

pub const PILL_WIDTH: f32 = 320.0;
pub const PILL_HEIGHT: f32 = 56.0;
pub const BOTTOM_MARGIN: f32 = 32.0;

fn pill_size() -> Size<Pixels> {
    Size {
        width: px(PILL_WIDTH),
        height: px(PILL_HEIGHT),
    }
}

fn bottom_center_bounds(screen_bounds: Bounds<Pixels>, size: Size<Pixels>) -> Bounds<Pixels> {
    Bounds {
        origin: point(
            screen_bounds.origin.x + (screen_bounds.size.width - size.width) / 2.0,
            screen_bounds.origin.y + screen_bounds.size.height - size.height - px(BOTTOM_MARGIN),
        ),
        size,
    }
}

/// Builds the overlay's `WindowOptions` for the given session. Wayland
/// compositors that support `wlr-layer-shell` (KDE, Hyprland, generic
/// wlroots) get a true pinned/always-on-top layer surface; everything else
/// (X11, macOS, and GNOME Wayland — which does not implement layer-shell at
/// all) falls back to a best-effort transparent `PopUp` window positioned at
/// the bottom-center of the primary display.
pub fn overlay_window_options(
    session: &DesktopSession,
    display_id: Option<DisplayId>,
    screen_bounds: Bounds<Pixels>,
) -> WindowOptions {
    let size = pill_size();

    let uses_layer_shell = matches!(
        session,
        DesktopSession::LinuxWayland { compositor } if compositor.supports_layer_shell()
    );

    if uses_layer_shell {
        #[cfg(target_os = "linux")]
        {
            use gpui::layer_shell::{Anchor, KeyboardInteractivity, LayerShellOptions};
            return WindowOptions {
                titlebar: None,
                app_id: Some("voxflow-overlay".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                focus: false,
                show: false,
                is_movable: false,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "voxflow-overlay".to_string(),
                    anchor: Anchor::BOTTOM,
                    margin: Some((px(0.), px(0.), px(BOTTOM_MARGIN), px(0.))),
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            };
        }
    }

    // X11, macOS, and GNOME/unknown-compositor Wayland fallback.
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bottom_center_bounds(
            screen_bounds,
            size,
        ))),
        display_id,
        titlebar: None,
        window_background: WindowBackgroundAppearance::Transparent,
        focus: false,
        show: false,
        is_movable: false,
        kind: WindowKind::PopUp,
        window_decorations: Some(WindowDecorations::Client),
        ..Default::default()
    }
}

/// Whether `session` is expected to only get the best-effort `PopUp`
/// fallback rather than a properly pinned layer-shell surface. Surfaced in
/// the settings UI as an explicit, documented limitation rather than a
/// silent degradation.
pub fn overlay_is_best_effort(session: &DesktopSession) -> bool {
    !matches!(
        session,
        DesktopSession::LinuxWayland { compositor } if compositor.supports_layer_shell()
    )
}
