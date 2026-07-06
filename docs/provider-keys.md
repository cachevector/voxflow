# BYOK Provider Keys

## OpenAI (default)

1. Create an API key at [platform.openai.com](https://platform.openai.com)
2. Add billing — transcription uses `gpt-4o-mini-transcribe` (~$0.003/min)
3. Enter key in VoxFlow Settings → General

## Local only

Set quality mode to **Offline** — no API key required. Local Whisper model download coming in settings.

## Cost estimate

| Usage | Approx cost (mini) |
|-------|-------------------|
| 10 min/day | ~₹75/mo |
| 30 min/day | ~₹225/mo |
| 45 min/day | ~₹336/mo |

VAD trimming reduces billable minutes by excluding silence.
