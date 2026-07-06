# VoxFlow

Blazing-fast native AI dictation for macOS, Linux & Windows. Local-first. BYOK. System-wide voice input.

**Speak anywhere. VoxFlow writes it for you.**

## Features (MVP)

- Global push-to-talk hotkey
- Floating bottom bar with calm state labels
- WebRTC VAD silence trimming
- BYOK Hybrid routing (local + OpenAI `gpt-4o-mini-transcribe`)
- Clipboard/accessibility text insertion
- SQLite history + cost dashboard
- Permission onboarding (macOS)
- Pro: cleanup, profiles, snippets, multi-provider stubs

## Quick start (macOS)

```bash
./scripts/build-macos.sh
./apps/macos/.build/release/VoxFlow
```

Hold **Left Control**, speak, release.

## Linux

```bash
cargo run -p voxflow-linux --release
```

Default hotkey: **Alt+Space** (toggle). See [docs/linux-wayland.md](docs/linux-wayland.md).

## Project structure

```
crates/          # Rust workspace (core pipeline)
apps/macos/      # SwiftUI menubar app
apps/linux-cosmic/
apps/windows/    # WinUI stub
website/         # Waitlist landing page
docs/
```

## Docs

- [Architecture](docs/architecture.md)
- [macOS setup](docs/macos-setup.md)
- [Linux / Wayland](docs/linux-wayland.md)
- [Performance budget](docs/performance-budget.md)
- [BYOK keys](docs/provider-keys.md)

## License

MIT — see product spec for commercial Pro tier plans.
