# VoxFlow Architecture

VoxFlow is a Rust-core native dictation app with platform-specific UI shells.

## Pipeline

```
Hotkey → Audio capture (CPAL) → VAD (WebRTC) → Provider router → Transcribe → Insert → History
```

## Crates

| Crate | Role |
|-------|------|
| `voxflow-core` | Dictation state machine, pipeline orchestration |
| `voxflow-audio` | CPAL capture on background thread |
| `voxflow-vad` | WebRTC VAD silence trimming |
| `voxflow-router` | Hybrid BYOK routing rules |
| `voxflow-transcribe` | OpenAI, local Whisper, Groq/Deepgram stubs |
| `voxflow-history` | SQLite transcripts + usage |
| `voxflow-cost` | Cost tracking, caps, projections |
| `voxflow-config` | Settings JSON + app profiles |
| `voxflow-insert` | Text insertion trait |
| `voxflow-ffi` | UniFFI exports for Swift/WinUI |

## Platform apps

- **macOS**: SwiftUI menubar app (`apps/macos`) via UniFFI
- **Linux**: Rust CLI/hotkey app (`apps/linux-cosmic`) with clipboard paste
- **Windows**: WinUI 3 shell stub (`apps/windows`)

## Default mode

BYOK Hybrid — local for short utterances, OpenAI `gpt-4o-mini-transcribe` for longer speech.
