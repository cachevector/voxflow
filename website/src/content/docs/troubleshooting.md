---
title: Troubleshooting
description: Fixes for the failures people actually hit — a dead hotkey, silent recordings, text that never arrives, and slow transcription.
group: Under the hood
order: 8
---

## The hotkey does nothing

Almost always Accessibility. VoxFlow needs it both to watch for <kbd>⌥</kbd> <kbd>⌃</kbd> and
to paste, and macOS gives no visible error when it is missing.

Check **System Settings → Privacy & Security → Accessibility**, then quit and relaunch
VoxFlow. If you have rebuilt the app since granting access, remove the stale entry and add it
again — see [Permissions](./permissions).

## Recording produces nothing

- Confirm Microphone access is granted, and that VoxFlow is listed. It only appears after the
  app has asked for it at least once.
- Check that the input device macOS is using is the one you are speaking into.
- Very short utterances can be trimmed away entirely by voice activity detection. Try again
  with a full sentence.

## Text is transcribed but never appears

Insertion works by putting text on the clipboard and sending a synthetic <kbd>⌘</kbd>
<kbd>V</kbd>. Some apps refuse synthetic input, and secure fields refuse it by design.

The transcript is still on your clipboard and still in history, so paste it yourself. If a
specific app fails consistently, it is worth
[opening an issue](https://github.com/cachevector/voxflow/issues) with the app named.

## Transcription is slow

- On Intel Macs there is no Metal acceleration, and Whisper is meaningfully slower. This is
  why Apple Silicon is recommended.
- Try a smaller Whisper model. Accuracy drops, but latency drops further.
- Check whether the delay is transcription or the rewrite call by running with
  `RUST_LOG=voxflow=info` and reading the stage timings — see
  [Performance budget](./performance).

## The rewrite pass fails

- Confirm the key is present in Keychain and correct for the provider you selected.
- For a Custom Endpoint, confirm the base URL is reachable from this machine right now. A
  home server over a VPN is only reachable while the VPN is up.
- On repeated failure the raw transcript is still inserted, so a broken rewrite provider
  degrades to plain dictation rather than losing your words.

## The build fails

- `whisper-rs` needs CMake. `brew install cmake`.
- Xcode Command Line Tools must be installed: `xcode-select --install`.
- Clear stale build state with `cargo clean` and rebuild if the workspace has been switched
  between branches.
