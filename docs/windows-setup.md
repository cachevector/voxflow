# Windows Development Setup

## Prerequisites

- Windows 10/11
- Rust stable (MSVC toolchain)
- Node.js + pnpm
- Tauri CLI (`cargo install tauri-cli` or `pnpm add -D @tauri-apps/cli`)
- [Tauri's Windows prerequisites](https://tauri.app/start/prerequisites/) (WebView2, Visual Studio C++ Build Tools)

## Build

```powershell
pnpm install
pnpm tauri dev      # dev mode, hot reload
pnpm tauri build    # release .msi/.exe
```

## Permissions

Windows has no formal runtime permission prompt for microphone access at the OS level the way macOS does, but Settings → Privacy & Security → Microphone can block app access — if recording fails, VoxFlow surfaces a clear error with a link to `ms-settings:privacy-microphone`.

Text insertion tries UI Automation first, then falls back to clipboard + simulated Ctrl+V — this fallback is the primary reliable path on Windows, since UI Automation tree coverage varies a lot by target application.

## Hotkey

Default: a modifier+key combo (configurable in Settings → Hotkey), registered via `tauri-plugin-global-shortcut`. Bare-modifier push-to-talk (e.g. hold Left Control) is available as an advanced option via a low-level keyboard hook.

## Unsigned builds

Dev builds are unsigned and will trigger Windows SmartScreen ("Windows protected your PC") on first run — click "More info" → "Run anyway". An Authenticode code-signing certificate is planned before any public distribution.

## API key

Set your transcription and rewrite provider keys in Settings → Providers. Keys are stored in Windows Credential Manager via the `keyring` crate, never in plaintext settings JSON.
