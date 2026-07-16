# macOS Development Setup

## Prerequisites

- macOS 13+
- Rust stable
- Node.js + pnpm
- Tauri CLI (`cargo install tauri-cli` or `pnpm add -D @tauri-apps/cli`)

## Build

```bash
pnpm install
pnpm tauri dev      # dev mode, hot reload
pnpm tauri build    # release .app/.dmg
```

Tauri packages a proper `.app` bundle so macOS can show the microphone permission prompt on first launch.

## Permissions

Grant in System Settings → Privacy & Security:

1. **Microphone** — VoxFlow appears in the list **after you launch the app once** and accept (or deny) the prompt. There is no manual "+" button on macOS.
2. **Accessibility** — direct text insertion (falls back to Cmd+V paste simulation if not granted)
3. **Input Monitoring** — only needed if you enable the advanced bare-modifier (hold Left Control) hotkey mode; the default combo-based hotkey doesn't require it

## Hotkey

Default: a modifier+key combo (configurable in Settings → Hotkey), avoiding the Input Monitoring requirement. Bare-modifier push-to-talk (e.g. hold Left Control) is available as an advanced option.

## API key

Set your transcription and rewrite provider keys in Settings → Providers. Keys are stored in the macOS Keychain via the `keyring` crate, never in plaintext settings JSON. Non-secret settings (provider kind, base URL, model, quality mode, etc.) live in `~/Library/Application Support/com.maskedsyntax.VoxFlow/settings.json`.
