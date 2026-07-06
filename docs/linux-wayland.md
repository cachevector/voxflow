# Linux / COSMIC / Wayland Notes

VoxFlow on Linux uses the shared Rust core with clipboard-first text insertion.

## Audio

- PipeWire via CPAL (default on modern distros)
- Run `voxflow` from `apps/linux-cosmic`

## Hotkeys

- Default: **Alt+Space** toggle (via `global-hotkey`)
- COSMIC global shortcuts may require manual binding in desktop settings

## Text insertion

**Primary path:** copy transcript to clipboard + `xdotool key ctrl+v` (X11).

On **Wayland**, automatic paste may not work in all apps. VoxFlow copies to clipboard and shows **Copied** state — paste manually with Ctrl+V.

## Secrets

API keys stored in settings JSON under XDG config dir (`~/.config/maskedsyntax/VoxFlow/`).

## Packaging

See `packaging/flatpak/com.maskedsyntax.VoxFlow.yml` for Flatpak manifest skeleton.

## Known limitations

- Global hotkeys vary by compositor
- Text injection is less reliable than macOS Accessibility APIs
- Be transparent with users — clipboard fallback is intentional
