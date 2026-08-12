use std::time::Duration;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

pub const OVERLAY_LABEL: &str = "overlay";
pub const MAIN_LABEL: &str = "main";

pub fn show_overlay(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        position_overlay_bottom_center(app, &window);
        let _ = window.show();
    }
}

fn position_overlay_bottom_center(app: &AppHandle, window: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let overlay_size = window.outer_size().unwrap_or(PhysicalSize::new(360, 64));
        let x = ((size.width as f64 / scale) - (overlay_size.width as f64 / scale)) / 2.0;
        let y = (size.height as f64 / scale) - (overlay_size.height as f64 / scale) - 48.0;
        let _ = window.set_position(PhysicalPosition::new(x.max(0.0) as i32, y.max(0.0) as i32));
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
