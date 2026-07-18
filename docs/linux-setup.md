# Linux setup

Covers building and running `voxflow-app` on Linux. Developed and validated
on Zorin OS 18.1 (Ubuntu-based); package names below are for `apt`-based
distros. See [phase0-spike-notes.md](phase0-spike-notes.md) for the
findings behind these requirements.

## Build dependencies

GPUI itself needs [Zed's own Linux dependency
list](https://github.com/zed-industries/zed/blob/main/script/linux), plus
two more that VoxFlow needs on top of it:

```sh
sudo apt-get update && sudo apt-get install -y \
  gcc g++ libasound2-dev libfontconfig-dev libgit2-dev libglib2.0-dev \
  libssl-dev libva-dev libvulkan1 libwayland-dev libx11-xcb-dev \
  libxkbcommon-x11-dev libzstd-dev make cmake clang lld llvm jq \
  gettext-base elfutils libsqlite3-dev build-essential pipewire \
  xdg-desktop-portal xdg-desktop-portal-gnome \
  libxdo-dev \
  libdbus-1-dev pkg-config
```

- `libxdo-dev` — required by `enigo`'s X11 backend (synthetic Ctrl+V for
  clipboard-paste insertion). Not part of Zed's own dependency list since
  Zed doesn't do synthetic input.
- `libdbus-1-dev` + `pkg-config` — required by `ksni` (the system tray
  icon's D-Bus binding, via `libdbus-sys`).

Rust toolchain: standard `rustup` install, stable channel. `gpui`,
`gpui_platform`, and `gpui_tokio` are pulled from a pinned `git` revision of
`zed-industries/zed` (not crates.io — see the comment above those
dependencies in `crates/voxflow-app/Cargo.toml`), since GPUI is pre-1.0
with routine breaking changes and the Wayland layer-shell support VoxFlow's
overlay needs postdates the last crates.io publish.

## Global hotkey

Default binding is **Super+Shift+Space** (`voxflow_platform::HotkeyBinding::default_binding()`).

- **X11 sessions**: works out of the box via `XGrabKey` (the `global-hotkey`
  crate). No configuration needed.
- **GNOME Wayland** (and any Wayland session without a working
  `GlobalShortcuts` portal — this includes Sway/`xdg-desktop-portal-wlr`,
  which doesn't implement that portal either): VoxFlow falls back to a
  manual binding. You bind a compositor keyboard shortcut yourself to run
  `voxflowctl trigger toggle`, and VoxFlow listens on a Unix socket
  (`$XDG_RUNTIME_DIR/voxflow.sock`) for it.

  GNOME setup: **Settings → Keyboard → Keyboard Shortcuts → Custom
  Shortcuts → +**, set the command to `voxflowctl trigger toggle` and pick
  your preferred key combo. Because GNOME's custom shortcuts only fire on
  key-press (not release), this uses **toggle mode**: one press starts
  listening, the next press stops and processes — not press-and-hold.

  Sway setup (true push-to-talk, since Sway delivers both press and
  release):
  ```
  bindsym $mod+space exec voxflowctl trigger down
  bindsym --release $mod+space exec voxflowctl trigger up
  ```

## Text insertion

- **X11**: clipboard write + synthetic Ctrl+V (`enigo`/XTest). Confirmed
  working end-to-end.
- **GNOME Wayland**: clipboard write only (`CopyOnlyInserter`) — the
  overlay shows "Copied — press Ctrl+V". Wayland blocks arbitrary synthetic
  input by design; a portal-based synthetic-paste path
  (`org.freedesktop.portal.RemoteDesktop`) is designed but not yet
  validated against a real GNOME session (see phase0-spike-notes.md, item
  0.7) — copy-only is the current default, not a placeholder.

## System tray

VoxFlow registers a `org.kde.StatusNotifierItem` D-Bus service (via
`ksni`), which is the standard modern tray-icon protocol. **On stock
GNOME, this will not be visually shown** — GNOME Shell dropped built-in
tray rendering years ago. To see the tray icon on GNOME, install the
**"AppIndicator and KStatusNotifierItem Support"** extension from
[extensions.gnome.org](https://extensions.gnome.org/extension/615/appindicator-support/).
KDE Plasma and most other desktop environments show it natively with no
extra steps.

## Secrets (API keys)

Stored via the OS keyring (`keyring` crate), which on Linux talks to the
Secret Service D-Bus API. Requires a running provider —
`gnome-keyring` (present by default on GNOME, and therefore on Zorin) or
`kwalletd` on KDE. On a minimal/tiling-WM-only setup without either
running, key storage will fail; this isn't yet handled with a graceful
fallback UX (tracked for the settings-UI work in Phase 2).

## Autostart

Not yet implemented (`voxflow_platform::AutostartBackend` trait exists;
the XDG `.desktop`-entry-based Linux implementation is upcoming work).

## Known limitations on GNOME Wayland

- Overlay window is a best-effort `PopUp` (not a pinned `wlr-layer-shell`
  surface — GNOME/Mutter doesn't implement that protocol at all). It may
  not stay reliably positioned/on-top across workspace switches.
- Hotkey requires the manual toggle-mode binding above (no true
  push-to-talk).
- Text insertion is copy-only (see above).

For full overlay/hotkey fidelity today, use GNOME on Xorg (X11 session), or
a Wayland compositor with `wlr-layer-shell` and portal support (KDE Plasma
6+, Hyprland).
