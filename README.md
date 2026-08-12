# VoxFlow

Wispr Flow–style dictation for **macOS** (Linux in progress): hold or toggle **Option+Ctrl**, speak, get local Whisper transcription, Groq-powered grammar/filler cleanup, and automatic paste at the cursor—with history for recovery.

## Stack

- **Tauri 2 + React** — settings window, bottom-center overlay pill, tray
- **Rust workspace** — CPAL audio, WebRTC VAD, whisper.cpp (`whisper-rs` + Metal), Groq chat cleanup, clipboard paste, SQLite history
- **macOS** — `CGEventTap` for Option+Ctrl; Accessibility for paste

## Quick start (macOS)

```bash
pnpm install
pnpm tauri dev
```

See [docs/macos-setup.md](docs/macos-setup.md) for permissions (Microphone, Accessibility) and [docs/linux-setup.md](docs/linux-setup.md) for Linux notes.

## Project structure

```
crates/          # Rust pipeline (audio, vad, whisper, provider, insert, history, platform, …)
src-tauri/       # Tauri host: hotkey tap, tray, commands
src/             # React UI (settings + overlay)
docs/
```

## License

MIT
