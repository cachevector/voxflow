//! Phase 0.8 spike: confirm the existing ClipboardPasteInserter mechanism
//! (arboard clipboard write + enigo synthetic Ctrl+V) still works correctly
//! on a plain X11 session. Click into any text field before running this —
//! it copies a marker string to the clipboard, waits 3s for you to make
//! sure the target field is focused, then simulates Ctrl+V.
use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let marker = "VoxFlowSpike: I made 10 cakes, no 12 cakes";
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(marker)?;
    println!("clipboard set to: {marker:?}");
    println!("click into a text field now — pasting in 3 seconds...");
    std::thread::sleep(Duration::from_secs(3));

    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.key(Key::Control, Direction::Press)?;
    enigo.key(Key::Unicode('v'), Direction::Click)?;
    enigo.key(Key::Control, Direction::Release)?;
    println!("sent synthetic Ctrl+V");
    Ok(())
}
