use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle,
};

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let open_settings =
        MenuItem::with_id(app, "open_settings", "Open Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit VoxFlow", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_settings, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open_settings" => crate::windows::show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}
