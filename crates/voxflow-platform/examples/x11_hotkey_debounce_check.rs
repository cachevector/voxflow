//! Verifies the autorepeat-debounce fix in X11Hotkey: a sustained hold
//! should now produce exactly one Pressed and one Released, not one pair
//! per autorepeat tick.

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("x11_hotkey_debounce_check only runs on Linux");
}

#[cfg(target_os = "linux")]
fn main() -> anyhow::Result<()> {
    use voxflow_platform::{GlobalHotkeyBackend, HotkeyBinding, X11Hotkey};

    tracing_subscriber::fmt::init();
    let backend = X11Hotkey::new().map_err(|e| anyhow::anyhow!(e))?;
    futures_lite::future::block_on(backend.register(HotkeyBinding::legacy_combo_binding()))
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("registered — hold Super+Shift+Space for a few seconds, listening for 30s");
    let rx = backend.events();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while std::time::Instant::now() < deadline {
        if let Ok(event) = rx.recv_blocking() {
            println!("EVENT: {event:?}");
        }
    }
    println!("done listening");
    Ok(())
}
