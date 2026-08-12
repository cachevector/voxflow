//! Optional Wayland paste via `org.freedesktop.portal.RemoteDesktop`.
//! When unavailable, callers should fall back to clipboard-only insertion.

#[cfg(target_os = "linux")]
pub async fn try_synthetic_paste() -> Result<(), String> {
    use ashpd::desktop::remote_desktop::RemoteDesktop;
    let proxy = RemoteDesktop::new()
        .await
        .map_err(|e| e.to_string())?;
    let _session = proxy
        .create_session()
        .await
        .map_err(|e| e.to_string())?;
    // Full key synthesis requires an active RemoteDesktop session with user
    // consent; until wired end-to-end, signal unavailability so the insert
    // layer uses copy-only fallback (documented Linux Wayland behavior).
    Err("RemoteDesktop portal paste not yet wired — use History to paste manually".into())
}

#[cfg(not(target_os = "linux"))]
pub async fn try_synthetic_paste() -> Result<(), String> {
    Err("not on Linux".into())
}
