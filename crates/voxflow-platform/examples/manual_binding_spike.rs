//! Phase 0.5 spike: confirm `voxflowctl trigger <action>` reliably delivers
//! events over the Unix socket that `ManualBindingHotkey` listens on. Run
//! this, then in another terminal run `voxflowctl trigger toggle` (or
//! `down`/`up`) a few times and watch the events print here.
use voxflow_platform::{GlobalHotkeyBackend, HotkeyBinding, ManualBindingHotkey};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let backend = ManualBindingHotkey::new();
    futures_lite::future::block_on(backend.register(HotkeyBinding {
        key_code: 0,
        modifiers: 0,
        label: "manual".into(),
    }))
    .map_err(|e| anyhow::anyhow!(e))?;

    println!(
        "listening on {} — run `voxflowctl trigger toggle` in another terminal",
        backend.socket_path().display()
    );

    let rx = backend.events();
    while let Ok(event) = rx.recv_blocking() {
        println!("received: {event:?}");
    }
    Ok(())
}
