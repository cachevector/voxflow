# macOS Development Setup

## Prerequisites

- macOS 13+
- Xcode 15+ / Swift 5.9+
- Rust stable

## Build

```bash
./scripts/build-macos.sh
open apps/macos/dist/VoxFlow.app
```

The build script packages a proper `.app` bundle so macOS can show the microphone permission prompt.

## Permissions

Grant in System Settings → Privacy & Security:

1. **Microphone** — VoxFlow appears in the list **after you launch the app once** and accept (or deny) the prompt. There is no manual "+" button on macOS.
2. **Accessibility** — Cmd+V paste insertion
3. **Input Monitoring** — global Left Control hotkey

## Hotkey

Default: hold **Left Control** (push-to-talk). Every Mac and external keyboard has this key.

## API key

Set OpenAI key in Settings → General, or in `~/Library/Application Support/com.maskedsyntax.VoxFlow/settings.json`.
