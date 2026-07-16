# Performance Budget

Targets from product spec (short utterances, 3–10 seconds):

| Stage | Target |
|-------|--------|
| Bar appears | < 80 ms |
| Recording starts | < 100 ms |
| End-of-speech → VAD done | < 250 ms |
| Short phrase transcription | < 800 ms (cloud) |
| End-to-insert total | < 1.5 s |

## Idle resources

- CPU: < 1–2% idle
- Recording: < 5–8% capture + VAD
- Memory: < 300–500 MB (Tauri + React shell, no in-process model loaded — MVP transcription/rewrite calls go out over the network to a configured cloud or self-hosted provider)

Since the UI layer is a WebView (Tauri), keep the hot path — hotkey handling, audio capture, VAD, state machine — in Rust. React only renders; it should never sit on the critical path for the bar-appears/recording-starts latency targets above.

## Instrumentation

`LatencyTracker` in `voxflow-core` records marks at each pipeline stage. Enable `RUST_LOG=voxflow=info` for latency reports.
