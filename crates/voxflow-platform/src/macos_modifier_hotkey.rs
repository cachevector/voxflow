//! Global Option+Control chord detection via `CGEventTap` (requires Accessibility).

use crate::hotkey::{GlobalHotkeyBackend, HotkeyBinding, HotkeyError, HotkeyEvent};
use async_trait::async_trait;
use core_foundation::runloop::CFRunLoop;
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    CGEventFlags,
};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn both_modifiers(flags: CGEventFlags) -> bool {
    flags.contains(CGEventFlags::CGEventFlagControl)
        && flags.contains(CGEventFlags::CGEventFlagAlternate)
}

pub struct MacModifierHotkey {
    tx: async_channel::Sender<HotkeyEvent>,
    rx: async_channel::Receiver<HotkeyEvent>,
    active: Arc<AtomicBool>,
}

impl Default for MacModifierHotkey {
    fn default() -> Self {
        Self::new()
    }
}

impl MacModifierHotkey {
    pub fn new() -> Self {
        let (tx, rx) = async_channel::unbounded();
        Self {
            tx,
            rx,
            active: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl GlobalHotkeyBackend for MacModifierHotkey {
    async fn register(&self, _binding: HotkeyBinding) -> Result<(), HotkeyError> {
        if self.active.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let tx = self.tx.clone();
        let active = self.active.clone();

        thread::Builder::new()
            .name("voxflow-hotkey-tap".into())
            .spawn(move || run_event_tap(tx, active))
            .map_err(|e| HotkeyError::Other(e.into()))?;

        Ok(())
    }

    async fn unregister(&self) -> Result<(), HotkeyError> {
        self.active.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn events(&self) -> async_channel::Receiver<HotkeyEvent> {
        self.rx.clone()
    }
}

fn run_event_tap(tx: async_channel::Sender<HotkeyEvent>, active: Arc<AtomicBool>) {
    let prev_both = Cell::new(false);

    let tap = match CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![
            CGEventType::FlagsChanged,
            CGEventType::KeyDown,
            CGEventType::KeyUp,
        ],
        move |_proxy, _event_type, event| {
            if !active.load(Ordering::SeqCst) {
                return Some(event.clone());
            }
            let both = both_modifiers(event.get_flags());
            let prev = prev_both.get();
            if both && !prev {
                let _ = tx.try_send(HotkeyEvent::Pressed);
            } else if !both && prev {
                let _ = tx.try_send(HotkeyEvent::Released);
            }
            prev_both.set(both);
            Some(event.clone())
        },
    ) {
        Ok(t) => t,
        Err(()) => {
            tracing::error!(
                "failed to create CGEventTap — grant Accessibility permission in System Settings"
            );
            return;
        }
    };

    let loop_source = tap
        .mach_port
        .create_runloop_source(0)
        .expect("runloop source");
    let run_loop = CFRunLoop::get_current();
    run_loop.add_source(
        &loop_source,
        unsafe { core_foundation::runloop::kCFRunLoopCommonModes },
    );
    tap.enable();
    CFRunLoop::run_current();
}
