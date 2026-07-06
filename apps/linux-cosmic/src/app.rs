use crate::insert::LinuxClipboardInserter;
use anyhow::{Context, Result};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use std::sync::Arc;
use std::time::Duration;
use voxflow_config::load_settings;
use voxflow_core::DictationEngine;
use voxflow_insert::InsertionBridge;

pub fn run() -> Result<()> {
    tracing::info!("VoxFlow Linux starting (COSMIC/Wayland-compatible clipboard mode)");

    let settings = load_settings().unwrap_or_default();
    let inserter = Arc::new(InsertionBridge::new(Box::new(LinuxClipboardInserter)));
    let engine = DictationEngine::new(settings, inserter).context("engine init")?;
    engine.prewarm()?;

    let manager = GlobalHotKeyManager::new().context("hotkey manager")?;
    let hotkey = HotKey::new(Some(Modifiers::ALT), Code::Space);
    manager.register(hotkey).context("register hotkey")?;

    tracing::info!("Hold Alt+Space to dictate. Clipboard fallback active on Wayland.");

    let mut recording = false;
    loop {
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id == hotkey.id() {
                if !recording {
                    recording = true;
                    let state = engine.on_hotkey_down();
                    tracing::info!(?state, "listening");
                } else {
                    recording = false;
                    match engine.on_hotkey_up(None) {
                        Ok(result) => {
                            tracing::info!(
                                latency_ms = result.latency_ms,
                                text = %result.text,
                                "dictation complete"
                            );
                        }
                        Err(e) => tracing::error!("dictation failed: {e}"),
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
