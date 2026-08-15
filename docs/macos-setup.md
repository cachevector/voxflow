# macOS Development Setup

Users should install from the
[DMG](https://github.com/cachevector/voxflow/releases/latest/download/VoxFlow-macos-arm64.dmg).
This page is the source-build path.

## Prerequisites

- macOS 13+ (Apple Silicon recommended for Metal-accelerated Whisper)
- Rust stable, Xcode Command Line Tools, `cmake` (for `whisper-rs`)
- Node.js 20+ and pnpm 9+
- Tauri CLI via devDependency (`pnpm tauri …`)

## Build

```bash
pnpm install
pnpm tauri dev
pnpm tauri build
```

## Permissions

System Settings → Privacy & Security:

1. **Microphone** — prompted on first capture; VoxFlow must be launched once before it appears in the list.
2. **Accessibility** — required for the global **Option+Ctrl** event tap and synthetic **Cmd+V** paste.

Open the onboarding panel in **Settings → General** or use **Hotkey → Open Accessibility settings**.

## Hotkey

Default: **Option+Ctrl** (hold or toggle — Settings → Hotkey). Uses `CGEventTap`, not a letter-key global shortcut.

## Groq API key

Settings → **AI Cleanup (Groq)**. Stored in Keychain under the `rewrite` secret ref. Local Whisper STT does not use Groq.

## Data paths

- Settings: `~/Library/Application Support/com.maskedsyntax.VoxFlow/settings.json`
- Whisper models: `~/Library/Application Support/com.maskedsyntax.VoxFlow/models/`
