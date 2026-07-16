# VoxFlow Architecture

VoxFlow is a Rust-core dictation app with a single Tauri v2 + React shell for Windows and macOS.

## Pipeline

```
Hotkey → Audio capture (CPAL) → VAD (WebRTC) → Provider router → Transcribe → AI rewrite → Insert → History
```

## Crates

| Crate | Role |
|-------|------|
| `voxflow-core` | Dictation state machine, pipeline orchestration |
| `voxflow-audio` | CPAL capture on background thread |
| `voxflow-vad` | WebRTC VAD silence trimming |
| `voxflow-router` | Hybrid BYOK routing rules |
| `voxflow-provider` | Generic OpenAI-compatible HTTP adapter (transcription + rewrite; OpenAI, Groq, or a self-hosted Custom Endpoint) |
| `voxflow-secrets` | Keyring-backed secret storage (macOS Keychain / Windows Credential Manager) |
| `voxflow-history` | SQLite transcripts + usage |
| `voxflow-cost` | Cost tracking, caps, projections |
| `voxflow-config` | Settings JSON + app profiles |
| `voxflow-insert` | Text insertion trait + macOS/Windows implementations |

## App shell

- `src-tauri/` — Tauri v2 Rust host: exposes commands (`invoke`) and events (`emit`) to the frontend, owns the hotkey listener and two windows (settings, floating overlay pill).
- `src/` — React + TypeScript frontend: settings pages and the floating waveform bar.

Windows and macOS only — Linux is not a target.

## Default mode

BYOK Hybrid — cheap/mini model first, higher-accuracy model when the router decides it's needed. A self-hosted Custom Endpoint (e.g. a Raspberry Pi reachable over Tailscale) is available as a first-class alternative for near-zero marginal cost.
