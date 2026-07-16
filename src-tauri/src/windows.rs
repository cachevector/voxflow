use std::time::Duration;
use tauri::{AppHandle, Manager};

pub const OVERLAY_LABEL: &str = "overlay";
pub const MAIN_LABEL: &str = "main";

pub fn show_overlay(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        let _ = window.show();
    }
}

pub fn hide_overlay(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        let _ = window.hide();
    }
}

/// Hides the overlay after a short delay so "Done"/"Copied" states are
/// visible for a beat before the pill disappears.
pub fn hide_overlay_after(app: AppHandle, delay: Duration) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(delay).await;
        hide_overlay(&app);
    });
}

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
