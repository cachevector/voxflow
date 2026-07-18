//! Phase 0.4 spike: does a GPUI Application survive with zero open windows,
//! or does the process exit as soon as the run closure returns without
//! having opened one? If it exits, voxflow-app needs a permanently-hidden
//! keep-alive window as a workaround for tray-icon-only background mode.
use gpui::prelude::*;
use gpui_platform::application;

fn main() {
    tracing_subscriber::fmt::init();
    application().run(|cx: &mut gpui::App| {
        gpui_tokio::init(cx);
        tracing::info!("run closure executed, zero windows opened, returning now");
        // Deliberately open no window at all and just return from the
        // closure — if the process exits right after this log line, GPUI
        // does not support background-only operation without a window.
        cx.spawn(async move |cx| {
            let mut i = 0;
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(1))
                    .await;
                i += 1;
                tracing::info!(seconds_alive = i, "still alive with zero windows");
                if i >= 8 {
                    std::process::exit(0);
                }
            }
        })
        .detach();
    });
}
