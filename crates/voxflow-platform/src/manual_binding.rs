use crate::hotkey::{GlobalHotkeyBackend, HotkeyBinding, HotkeyError, HotkeyEvent};
use async_trait::async_trait;
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// The GNOME Wayland / Sway fallback hotkey backend. Rather than grabbing a
/// key itself (impossible on GNOME Wayland, and Sway has no
/// `GlobalShortcuts` portal either), this listens on a Unix socket and
/// expects the user's compositor to invoke `voxflowctl trigger <action>` on
/// key press (and, where the compositor supports it, release):
///
/// - Sway: `bindsym $mod+space exec voxflowctl trigger down` /
///   `bindsym --release $mod+space exec voxflowctl trigger up` — true
///   push-to-talk, since Sway delivers both press and release.
/// - GNOME: Custom Shortcuts only fire on press, so bind a single shortcut
///   to `voxflowctl trigger toggle` and use `DictationMode::Toggle`.
pub struct ManualBindingHotkey {
    socket_path: PathBuf,
    tx: async_channel::Sender<HotkeyEvent>,
    rx: async_channel::Receiver<HotkeyEvent>,
    running: Arc<AtomicBool>,
}

impl ManualBindingHotkey {
    pub fn new() -> Self {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
        let socket_path = PathBuf::from(runtime_dir).join("voxflow.sock");
        let (tx, rx) = async_channel::unbounded();
        Self {
            socket_path,
            tx,
            rx,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    fn handle_client(stream: UnixStream, tx: &async_channel::Sender<HotkeyEvent>) {
        let reader = BufReader::new(stream);
        for line in reader.lines().map_while(Result::ok) {
            let event = match line.trim() {
                "down" => Some(HotkeyEvent::Pressed),
                "up" => Some(HotkeyEvent::Released),
                "toggle" => Some(HotkeyEvent::Toggled),
                other => {
                    tracing::warn!(action = other, "unrecognized voxflowctl trigger action");
                    None
                }
            };
            if let Some(event) = event {
                let _ = tx.send_blocking(event);
            }
        }
    }
}

impl Default for ManualBindingHotkey {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GlobalHotkeyBackend for ManualBindingHotkey {
    async fn register(&self, _binding: HotkeyBinding) -> Result<(), HotkeyError> {
        // The binding itself lives in the user's compositor config, not
        // here — this just needs to start listening.
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(()); // already listening
        }

        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).map_err(|e| {
                HotkeyError::Other(anyhow::anyhow!(
                    "removing stale socket at {}: {e}",
                    self.socket_path.display()
                ))
            })?;
        }

        let listener = UnixListener::bind(&self.socket_path).map_err(|e| {
            HotkeyError::Other(anyhow::anyhow!(
                "binding {}: {e}",
                self.socket_path.display()
            ))
        })?;

        let tx = self.tx.clone();
        let running = self.running.clone();
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                match conn {
                    Ok(stream) => Self::handle_client(stream, &tx),
                    Err(e) => tracing::warn!(error = %e, "voxflow socket accept error"),
                }
            }
        });

        tracing::info!(path = %self.socket_path.display(), "listening for voxflowctl trigger events");
        Ok(())
    }

    async fn unregister(&self) -> Result<(), HotkeyError> {
        self.running.store(false, Ordering::SeqCst);
        let _ = std::fs::remove_file(&self.socket_path);
        Ok(())
    }

    fn events(&self) -> async_channel::Receiver<HotkeyEvent> {
        self.rx.clone()
    }
}
