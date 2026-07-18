//! Phase 0.6 spike: confirm `global-hotkey` registers and delivers
//! press/release events on this X11 session. Registers Super+Shift+Space
//! (VoxFlow's documented default binding) and prints every event for 20s.
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let manager = GlobalHotKeyManager::new()?;
    let hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Space);
    manager.register(hotkey)?;
    println!(
        "registered Super+Shift+Space (id={}) — press it now, listening for 40s",
        hotkey.id()
    );

    let receiver = GlobalHotKeyEvent::receiver();
    let deadline = std::time::Instant::now() + Duration::from_secs(40);
    while std::time::Instant::now() < deadline {
        if let Ok(event) = receiver.recv_timeout(Duration::from_millis(200)) {
            println!("EVENT: id={} state={:?}", event.id(), event.state());
        }
    }
    println!("done listening");
    Ok(())
}
