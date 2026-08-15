---
title: Architecture
description: How the VoxFlow Rust workspace is laid out — the crates, the pipeline they form, and how the Tauri shell sits on top of them.
group: Under the hood
order: 6
---

VoxFlow is a Rust core with a Tauri 2 + React shell on top. The split is deliberate: anything
with a latency target lives in Rust, and the web layer only renders state that has already
been decided.

## The pipeline

```
Hotkey → Audio capture (CPAL) → VAD (WebRTC) → Whisper (local)
       → AI rewrite → Insert at cursor → History
```

## Crates

| Crate | Role |
|---|---|
| `voxflow-core` | Dictation state machine and pipeline orchestration |
| `voxflow-audio` | CPAL capture on a background thread |
| `voxflow-vad` | WebRTC voice activity detection, silence trimming |
| `voxflow-whisper` | Local transcription via `whisper-rs` with Metal |
| `voxflow-provider` | Generic OpenAI-compatible HTTP adapter for the rewrite pass |
| `voxflow-router` | Routing rules across configured providers |
| `voxflow-insert` | Text insertion trait and platform implementations |
| `voxflow-history` | SQLite transcripts and usage records |
| `voxflow-cost` | Cost tracking, caps, projections |
| `voxflow-config` | Settings JSON and per-app profiles |
| `voxflow-secrets` | Keyring-backed secret storage |
| `voxflow-platform` | Platform-specific glue |
| `voxflowctl` | Command-line utility, including key management |

## The app shell

- `src-tauri/` — the Tauri host. Owns the hotkey listener, the tray icon, and two windows: a
  hidden settings window and a transparent always-on-top overlay. Exposes commands to the
  frontend through `invoke` and pushes state to it through events.
- `src/` — React and TypeScript. The settings pages, and the overlay pill with its live
  waveform.

The overlay receives amplitude samples as events and draws them. It is never asked to make a
decision, because a WebView cannot be trusted with an 80 millisecond budget.

## Design rules

The product spec sets constraints the implementation is held to:

- Never steal focus unless it is unavoidable.
- Never open a large window during dictation.
- Never make the user paste manually unless insertion genuinely failed.
- Always give a fallback path.
- Always prioritise perceived speed.
