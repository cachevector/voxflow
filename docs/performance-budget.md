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
- Memory: < 300–500 MB with tiny local model

## Instrumentation

`LatencyTracker` in `voxflow-core` records marks at each pipeline stage. Enable `RUST_LOG=voxflow=info` for latency reports.
