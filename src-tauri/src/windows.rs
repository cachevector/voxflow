use std::time::Duration;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

pub const OVERLAY_LABEL: &str = "overlay";
pub const MAIN_LABEL: &str = "main";
const OVERLAY_BOTTOM_MARGIN: f64 = 48.0;

pub fn show_overlay(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        position_overlay_bottom_center(app, &window);
        let _ = window.show();
    }
}

fn position_overlay_bottom_center(app: &AppHandle, window: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let work_area = monitor.work_area();
        let overlay_size = window.outer_size().unwrap_or(PhysicalSize::new(360, 64));
        let bottom_margin = (OVERLAY_BOTTOM_MARGIN * monitor.scale_factor()).round() as u32;
        let position = bottom_center_position(
            work_area.position,
            work_area.size,
            overlay_size,
            bottom_margin,
        );
        let _ = window.set_position(position);
    }
}

fn bottom_center_position(
    work_area_position: PhysicalPosition<i32>,
    work_area_size: PhysicalSize<u32>,
    overlay_size: PhysicalSize<u32>,
    bottom_margin: u32,
) -> PhysicalPosition<i32> {
    let x_offset = work_area_size.width.saturating_sub(overlay_size.width) / 2;
    let y_offset = work_area_size
        .height
        .saturating_sub(overlay_size.height)
        .saturating_sub(bottom_margin);

    PhysicalPosition::new(
        work_area_position.x.saturating_add_unsigned(x_offset),
        work_area_position.y.saturating_add_unsigned(y_offset),
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_overlay_above_a_small_dock() {
        let position = bottom_center_position(
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(3024, 1814),
            PhysicalSize::new(720, 128),
            96,
        );

        assert_eq!(position, PhysicalPosition::new(1152, 1590));
        assert_eq!(position.y + 128 + 96, 1814);
    }

    #[test]
    fn positions_overlay_above_a_large_dock() {
        let position = bottom_center_position(
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(3024, 1510),
            PhysicalSize::new(720, 128),
            96,
        );

        assert_eq!(position, PhysicalPosition::new(1152, 1286));
        assert_eq!(position.y + 128 + 96, 1510);
    }

    #[test]
    fn includes_the_work_area_origin_for_offset_monitors_and_side_docks() {
        let position = bottom_center_position(
            PhysicalPosition::new(-1600, 48),
            PhysicalSize::new(1500, 900),
            PhysicalSize::new(360, 64),
            48,
        );

        assert_eq!(position, PhysicalPosition::new(-1030, 836));
    }
}
