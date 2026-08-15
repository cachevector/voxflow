---
title: Performance budget
description: The latency and resource targets VoxFlow is built against, and how they are measured.
group: Under the hood
order: 7
---

Dictation is only useful if it feels instant. These are the numbers the implementation is
held to, not aspirations.

## Latency

For a short utterance of three to ten seconds:

| Stage | Target |
|---|---|
| Pill appears | < 80 ms |
| Recording starts | < 100 ms |
| End of speech → VAD done | < 250 ms |
| Short phrase transcription | < 800 ms |
| End of speech → text inserted | < 1.5 s |

## Resources

| Condition | Target |
|---|---|
| Idle CPU | 1–2% |
| Recording CPU | 5–8% for capture and VAD |
| Memory | 300–500 MB for the shell, before a Whisper model is loaded |

A loaded Whisper model adds its own footprint on top, which depends on the model size you
choose.

## Keeping the hot path fast

Because the interface is a WebView, everything with a deadline stays in Rust: the hotkey
handler, audio capture, VAD, and the state machine. React is never on the critical path for
the "pill appears" or "recording starts" targets — by the time it draws anything, the
decision has already been made.

## Measuring it

`LatencyTracker` in `voxflow-core` records a mark at each pipeline stage. Turn on the reports
with:

```bash
RUST_LOG=voxflow=info pnpm tauri dev
```

Each dictation then logs its stage timings, so a regression shows up as a specific stage
rather than a vague feeling that the app got slower.
