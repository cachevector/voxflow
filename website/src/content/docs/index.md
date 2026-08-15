---
title: Introduction
description: What VoxFlow is, what it does to your voice on the way to your cursor, and what state the project is in today.
group: Start here
order: 1
---

VoxFlow is a voice dictation app for macOS. You hold a hotkey, speak, and let go. A moment
later the text is sitting at your cursor in whatever app had focus — cleaned up, punctuated,
and free of the "um"s.

It is built for people who want that without a subscription and without shipping their voice
to someone else's server. Transcription runs on your own machine. Provider keys are yours.
The whole thing is MIT licensed.

## What happens when you dictate

1. **Hotkey.** Hold <kbd>⌥</kbd> <kbd>⌃</kbd>. A small pill fades in at the bottom of the
   screen. Nothing takes focus.
2. **Capture.** Audio comes in through CoreAudio and passes a voice activity detector, which
   trims the silence at either end.
3. **Transcribe.** Whisper runs locally through `whisper.cpp` with Metal acceleration.
4. **Rewrite.** The raw transcript goes through an AI cleanup pass that turns speech into a
   sentence. This is the one step that can leave your machine, and you choose where it goes.
5. **Insert.** The finished text is pasted at the cursor, and stored in a local history
   database so you can recover it later.

## What it is built on

| Layer | Technology |
|---|---|
| Core pipeline | Rust workspace — audio, VAD, Whisper, provider, insertion, history |
| Audio capture | CPAL (CoreAudio) |
| Silence trimming | WebRTC VAD |
| Transcription | `whisper-rs` / `whisper.cpp` with Metal |
| Rewrite pass | Any OpenAI-compatible endpoint — Groq, OpenAI, or self-hosted |
| App shell | Tauri 2 + React — settings window, overlay pill, tray |
| History | SQLite |
| Secrets | macOS Keychain via the `keyring` crate |

Everything on the latency-critical path — the hotkey tap, capture, VAD, the state machine —
is Rust. The web layer only draws.

## Project status

VoxFlow is in early development. The supported way to install it is the
[macOS disk image](./install). Expect a few rough edges.

- **macOS** is the supported platform today. macOS 13 or later, Apple Silicon recommended so
  Whisper can use Metal.
- **Linux** support is in progress. The Wayland and X11 differences around global hotkeys and
  synthetic input are real and not yet resolved.
- **Windows** is not a target.

Start with [Install on macOS](./install).
