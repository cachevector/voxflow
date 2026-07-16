# VoxFlow

Fast AI dictation for macOS & Windows, built on Rust + Tauri + React. BYOK — bring your own cloud key, or point it at a fully self-hosted server. System-wide voice input with a built-in AI rewrite pass.

**Speak anywhere. VoxFlow writes it for you.**

## Features (MVP)

- Global push-to-talk hotkey
- Floating bottom bar with a live waveform and calm state labels
- WebRTC VAD silence trimming
- BYOK Hybrid routing (OpenAI/Groq cloud, or a self-hosted Custom Endpoint — e.g. a Raspberry Pi over Tailscale)
- AI rewrite pass on by default — every transcript is cleaned up into a polished sentence, independently configurable from transcription
- Accessibility/UI-Automation text insertion with clipboard fallback
- SQLite history + cost dashboard
- Permission onboarding (macOS + Windows)
- Keys stored in the OS keychain/credential manager, never plaintext

## Quick start

```bash
pnpm install
pnpm tauri dev
```

See [docs/macos-setup.md](docs/macos-setup.md) or [docs/windows-setup.md](docs/windows-setup.md) for platform-specific prerequisites and permissions.

## Project structure

```
crates/          # Rust workspace (core pipeline: audio, VAD, router, provider, insert, history, cost, config, secrets)
src-tauri/       # Tauri v2 app: commands, events, hotkey, tray, window management
src/             # React + TypeScript frontend: settings window + floating overlay bar
website/         # Waitlist landing page
docs/
```

## Docs

- [Rewrite plan & current status](docs/REWRITE_PLAN.md) — start here to see what's done and what's next
- [Linux migration plan](docs/LINUX_MIGRATION_PLAN.md) — approved, not yet executed: swaps Windows for Linux (Wayland-first, X11 secondary)
- [Architecture](docs/architecture.md)
- [macOS setup](docs/macos-setup.md)
- [Windows setup](docs/windows-setup.md)
- [Performance budget](docs/performance-budget.md)
- [BYOK keys](docs/provider-keys.md)

## License

MIT — see product spec for commercial Pro tier plans.
