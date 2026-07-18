use crate::hotkey::{GlobalHotkeyBackend, HotkeyBinding, HotkeyError, HotkeyEvent};
use async_trait::async_trait;
use crossbeam_channel::RecvTimeoutError;
use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use parking_lot::Mutex;
use std::time::Duration;

/// X11 key auto-repeat means a sustained hold of a grabbed key produces
/// repeated Released/Pressed pairs, not one continuous press — confirmed
/// empirically (a 3s hold produced 3 full press/release pairs). This is a
/// known quirk of `global-hotkey`'s X11 backend (it uses its own internal
/// Xlib connection we have no access to, so we can't fix it by enabling
/// XKB detectable-autorepeat on a different connection — that setting is
/// per-connection). Instead: a Released is held back briefly; if a matching
/// Pressed arrives within this window, both are swallowed as one
/// continuous hold. Default X11 autorepeat is ~25-30Hz (33-40ms between
/// repeats), so 80ms comfortably absorbs repeat noise without adding
/// perceptible latency to a genuine key release.
const AUTOREPEAT_DEBOUNCE: Duration = Duration::from_millis(80);

/// X11 (and XWayland) push-to-talk hotkey, backed by `XGrabKey` via the
/// `global-hotkey` crate. Delivers true press/release pairs — confirmed by
/// the Phase 0.6 spike against a real X11 session.
///
/// Note: `global-hotkey`'s event stream (`GlobalHotKeyEvent::receiver()`) is
/// a single process-wide static channel, not scoped to a manager instance —
/// only one `X11Hotkey` should be constructed per process.
pub struct X11Hotkey {
    manager: GlobalHotKeyManager,
    registered: Mutex<Option<HotKey>>,
    tx: async_channel::Sender<HotkeyEvent>,
    rx: async_channel::Receiver<HotkeyEvent>,
}

impl X11Hotkey {
    pub fn new() -> Result<Self, HotkeyError> {
        let manager = GlobalHotKeyManager::new().map_err(|e| HotkeyError::Other(e.into()))?;
        let (tx, rx) = async_channel::unbounded();
        Ok(Self {
            manager,
            registered: Mutex::new(None),
            tx,
            rx,
        })
    }
}

#[async_trait]
impl GlobalHotkeyBackend for X11Hotkey {
    async fn register(&self, binding: HotkeyBinding) -> Result<(), HotkeyError> {
        let hotkey = HotKey::new(Some(binding.modifiers), binding.code);
        self.manager
            .register(hotkey)
            .map_err(|e| HotkeyError::Other(e.into()))?;
        *self.registered.lock() = Some(hotkey);

        let expected_id = hotkey.id();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let receiver = GlobalHotKeyEvent::receiver();
            // No release forwarded yet, so wait indefinitely; a day is a
            // practical stand-in for "no timeout" with recv_timeout's API.
            const IDLE_WAIT: Duration = Duration::from_secs(60 * 60 * 24);
            let mut pending_release = false;

            loop {
                let wait = if pending_release {
                    AUTOREPEAT_DEBOUNCE
                } else {
                    IDLE_WAIT
                };
                match receiver.recv_timeout(wait) {
                    Ok(event) if event.id() == expected_id => match event.state() {
                        HotKeyState::Pressed => {
                            if !pending_release {
                                if tx.send_blocking(HotkeyEvent::Pressed).is_err() {
                                    break;
                                }
                            }
                            // else: auto-repeat during a hold — the pending
                            // Released was never forwarded, so from the
                            // caller's perspective this is still one
                            // continuous press.
                            pending_release = false;
                        }
                        HotKeyState::Released => pending_release = true,
                    },
                    Ok(_) => {} // another hotkey registered elsewhere in-process
                    Err(RecvTimeoutError::Timeout) => {
                        if pending_release {
                            pending_release = false;
                            if tx.send_blocking(HotkeyEvent::Released).is_err() {
                                break;
                            }
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        tracing::info!(binding = %binding.label, "registered X11 global hotkey");
        Ok(())
    }

    async fn unregister(&self) -> Result<(), HotkeyError> {
        if let Some(hotkey) = self.registered.lock().take() {
            self.manager
                .unregister(hotkey)
                .map_err(|e| HotkeyError::Other(e.into()))?;
        }
        Ok(())
    }

    fn events(&self) -> async_channel::Receiver<HotkeyEvent> {
        self.rx.clone()
    }
}
