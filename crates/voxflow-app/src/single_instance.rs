use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

/// Ensures only one `voxflow-app` runs at a time, mirroring
/// `tauri-plugin-single-instance`'s role in the old shell. Uses a dedicated
/// socket (separate from `ManualBindingHotkey`'s trigger socket, which has
/// no liveness check and would silently steal a stale path rather than
/// reject a live second instance).
///
/// Distinguishes "stale socket file from an unclean shutdown" (safe to
/// remove and rebind) from "another instance is genuinely running" (must
/// exit) by attempting to connect before removing anything.
pub fn acquire_or_exit() -> UnixListener {
    let path = lock_path();

    if path.exists() {
        match UnixStream::connect(&path) {
            Ok(_) => {
                eprintln!("voxflow-app is already running (lock at {})", path.display());
                std::process::exit(1);
            }
            Err(_) => {
                // Stale socket left behind by an unclean shutdown.
                let _ = std::fs::remove_file(&path);
            }
        }
    }

    UnixListener::bind(&path).unwrap_or_else(|e| {
        eprintln!("failed to acquire single-instance lock at {}: {e}", path.display());
        std::process::exit(1);
    })
}

fn lock_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(runtime_dir).join("voxflow-instance.sock")
}
