# BYOK Provider Keys

VoxFlow uses one generic OpenAI-API-compatible adapter for both the transcription call and the AI rewrite call. Each can be configured independently in Settings → Transcription / Settings → AI Rewrite. Keys are stored in the OS keychain (macOS Keychain / Windows Credential Manager) via the `keyring` crate — never in plaintext settings JSON.

## OpenAI (default)

1. Create an API key at [platform.openai.com](https://platform.openai.com)
2. Add billing — transcription uses `gpt-4o-mini-transcribe` (~$0.003/min)
3. Enter key in VoxFlow Settings → Providers

## Groq

Same adapter, different base URL (`https://api.groq.com/openai/v1`) and key. Useful as a fast/cheap alternative for either the transcription or rewrite call.

## Custom Endpoint (self-hosted)

Point VoxFlow at any OpenAI-API-compatible server you run yourself — for example `whisper.cpp-server`/`faster-whisper-server` for transcription and `llama.cpp-server`/Ollama (OpenAI-compat mode) for the rewrite pass, running on a Raspberry Pi or other machine you own. Set the provider kind to **Custom Endpoint**, enter the base URL (e.g. your Tailscale address like `http://raspberrypi.tailnet-name.ts.net:8080/v1`), and an API key only if your server requires one. This drives marginal cost to roughly $0 — you're paying for hardware/electricity you already have, not per-minute API billing. Reachability while away from home depends on a VPN/tunnel (Tailscale or similar) you set up yourself; VoxFlow just calls whatever base URL is configured.

## Cost estimate (cloud)

| Usage | Approx cost (mini) |
|-------|-------------------|
| 10 min/day | ~₹75/mo |
| 30 min/day | ~₹225/mo |
| 45 min/day | ~₹336/mo |

VAD trimming reduces billable minutes by excluding silence. These estimates don't apply to a Custom Endpoint setup, where marginal cost is near-zero.
