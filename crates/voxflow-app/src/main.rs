mod app_state;
mod hotkey_glue;
mod overlay;
mod single_instance;
mod tray;

use app_state::AppState;
use gpui::prelude::*;
use gpui_platform::application;
use overlay::{overlay_is_best_effort, overlay_window_options, OverlayView};
use std::sync::Arc;
use std::time::Duration;
use voxflow_core::{StateEvent, UiState};
use voxflow_platform::HotkeyBinding;

const HIDE_DELAY: Duration = Duration::from_millis(900);

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    // Held for the process lifetime: dropping it releases the lock socket.
    let _instance_lock = single_instance::acquire_or_exit();

    let state = Arc::new(AppState::new()?);
    tracing::info!(session = ?state.session, "voxflow-app starting");
    if overlay_is_best_effort(&state.session) {
        tracing::warn!(
            "overlay window uses the best-effort PopUp fallback on this session (no wlr-layer-shell support) — position/always-on-top is not guaranteed"
        );
    }

    let (overlay_tx, overlay_rx) = async_channel::unbounded::<StateEvent>();

    let hotkey = state.hotkey.clone();
    futures_lite::future::block_on(hotkey.register(HotkeyBinding::default_binding()))
        .map_err(|e| anyhow::anyhow!(e))?;

    hotkey_glue::spawn(state.clone(), overlay_tx);
    tray::spawn();

    application().run(move |cx: &mut gpui::App| {
        gpui_tokio::init(cx);
        // GPUI's default quit_mode is LastWindowClosed on Linux (mirroring
        // typical desktop-app conventions) — that's wrong for a background
        // dictation daemon whose only window is the overlay, which is
        // supposed to open and close repeatedly without ending the process.
        cx.set_quit_mode(gpui::QuitMode::Explicit);
        let session = state.session.clone();

        cx.spawn(async move |cx| {
            let mut window: Option<gpui::WindowHandle<OverlayView>> = None;

            while let Ok(event) = overlay_rx.recv().await {
                let terminal = matches!(
                    event.ui_state,
                    UiState::Copied | UiState::Done | UiState::Error
                );

                if let Some(handle) = window {
                    if handle.update(cx, |view, _, cx| {
                        view.event = event;
                        cx.notify();
                    }).is_err() {
                        window = None;
                    } else {
                        window = Some(handle);
                    }
                } else {
                    let opened = cx.update(|cx| {
                        let display = cx.primary_display();
                        let display_id = display.as_ref().map(|d| d.id());
                        let screen_bounds =
                            display.map(|d| d.bounds()).unwrap_or_else(|| gpui::Bounds {
                                origin: gpui::point(gpui::px(0.), gpui::px(0.)),
                                size: gpui::size(gpui::px(1920.), gpui::px(1080.)),
                            });
                        let mut options =
                            overlay_window_options(&session, display_id, screen_bounds);
                        options.show = true;
                        cx.open_window(options, |_, cx| cx.new(|_| OverlayView { event }))
                    });
                    window = opened.ok();
                }

                if terminal {
                    cx.background_executor().timer(HIDE_DELAY).await;
                    if let Some(handle) = window.take() {
                        let _ = handle.update(cx, |_, window, _| window.remove_window());
                    }
                }
            }
        })
        .detach();
    });

    Ok(())
}
