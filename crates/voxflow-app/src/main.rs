mod overlay;

use gpui::prelude::*;
use gpui_platform::application;
use overlay::{overlay_is_best_effort, overlay_window_options, OverlayView};

fn main() {
    tracing_subscriber::fmt::init();
    let session = voxflow_platform::detect_session();
    tracing::info!(?session, "voxflow-app starting (Phase 0 overlay spike)");

    application().run(move |cx: &mut gpui::App| {
        // GPUI's Linux backend uses zbus internally (portal/DE integration)
        // and panics without a Tokio reactor present on the thread it runs
        // on — see Phase 0 spike notes. Must be initialized before anything
        // else touches those code paths.
        gpui_tokio::init(cx);

        let display = cx.primary_display();
        let display_id = display.as_ref().map(|d| d.id());
        let screen_bounds = display
            .map(|d| d.bounds())
            .unwrap_or_else(|| gpui::Bounds {
                origin: gpui::point(gpui::px(0.), gpui::px(0.)),
                size: gpui::size(gpui::px(1920.), gpui::px(1080.)),
            });

        let mut options = overlay_window_options(&session, display_id, screen_bounds);
        options.show = true; // spike: show immediately instead of the real hide/show-on-hotkey flow

        let best_effort = overlay_is_best_effort(&session);
        let label = if best_effort {
            format!("VoxFlow (best-effort PopUp — {session:?})")
        } else {
            format!("VoxFlow (layer-shell — {session:?})")
        };

        cx.open_window(options, |_, cx| cx.new(|_| OverlayView { label }))
            .expect("failed to open overlay spike window");

        cx.activate(true);
    });
}
