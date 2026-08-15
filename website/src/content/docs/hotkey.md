---
title: Hotkey and the overlay
description: How the Option+Ctrl hotkey works in hold and toggle modes, why VoxFlow uses a modifier tap instead of a letter shortcut, and what the overlay pill is telling you.
group: Using VoxFlow
order: 4
---

The default binding is <kbd>⌥</kbd> <kbd>⌃</kbd> — Option and Control together, with no letter
key.

## Why a modifier combination

Most dictation tools bind a letter shortcut, which means eventually colliding with something
inside the app you are writing in. VoxFlow instead installs a `CGEventTap` and watches the
modifier flags directly. Nothing is swallowed from the focused app, and there is no letter
left to conflict with.

The cost of this approach is that it requires Accessibility access — see
[Permissions](./permissions).

## Hold or toggle

Both modes are available under **Settings → Hotkey**.

- **Hold** — press and hold to record, release to finish. Best for short phrases, and the
  mode the latency budget is tuned around.
- **Toggle** — press once to start, once again to stop. Better for long passages where
  holding the keys gets tiring.

Voice activity detection trims silence at both ends either way, so a pause before you start
talking costs you nothing.

## Reading the overlay

While you dictate, a small pill sits at the bottom of the screen. It never takes focus and
never covers what you are working on.

| State | What it means |
|---|---|
| Cyan mark, live waveform | Listening. The waveform tracks your actual microphone level. |
| Dimmed mark | Speech ended. Transcribing and rewriting. |
| Pill disappears | Text has been inserted at your cursor. |

If insertion fails — usually because the target app rejects synthetic paste — the text is
still on your clipboard and still in history, so nothing is lost.

## Latency you should expect

These are the targets the pipeline is built against, for a short utterance of three to ten
seconds:

| Stage | Target |
|---|---|
| Pill appears | under 80 ms |
| Recording starts | under 100 ms |
| End of speech to VAD done | under 250 ms |
| Short phrase transcribed | under 800 ms |
| End of speech to inserted text | under 1.5 s |
