---
title: Permissions
description: The two macOS permissions VoxFlow needs — Microphone and Accessibility — why each is required, and how to fix them when they go wrong.
group: Start here
order: 3
---

VoxFlow needs two permissions from macOS. Both are granted in **System Settings → Privacy &
Security**, and the app cannot function without the second one.

## Microphone

Required to capture audio. macOS prompts for it the first time VoxFlow tries to record.

VoxFlow has to be launched at least once before it appears in the Microphone list — macOS
only lists apps that have actually asked.

## Accessibility

Required for two separate things:

- **The global hotkey.** VoxFlow watches for <kbd>⌥</kbd> <kbd>⌃</kbd> with a `CGEventTap`,
  which macOS only permits for apps with Accessibility access.
- **Inserting text.** Pasting at your cursor uses a synthetic <kbd>⌘</kbd> <kbd>V</kbd>, which
  is also gated behind Accessibility.

Without Accessibility, the hotkey silently does nothing. This is the single most common
reason VoxFlow appears not to work.

Grant it in **System Settings → Privacy & Security → Accessibility**, then toggle VoxFlow on.
The in-app shortcut is **Settings → General**, or **Hotkey → Open Accessibility settings**.

> After granting or re-granting Accessibility, quit and relaunch VoxFlow. macOS does not
> reliably apply the change to a running process.

## When a permission stops working

Rebuilding the app changes its binary, and macOS sometimes keeps a stale Accessibility entry
pointing at the old one. If the hotkey stops responding after a rebuild:

1. Open **System Settings → Privacy & Security → Accessibility**.
2. Select VoxFlow and remove it with the **−** button.
3. Relaunch VoxFlow and add it again.

The same applies to Microphone access if recording starts producing silence.
