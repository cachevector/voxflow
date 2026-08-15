---
title: Install on macOS
description: Build and run VoxFlow from source on macOS, including prerequisites, the dev loop, and where the app stores its data.
group: Start here
order: 2
sidebarLabel: Install
---

VoxFlow has no signed release build yet, so you build it from source. This takes a few
minutes the first time — `whisper.cpp` and the Rust workspace both need compiling.

## Prerequisites

- **macOS 13 or later.** Apple Silicon is recommended: it is what makes Metal-accelerated
  Whisper fast enough to stay inside the latency budget.
- **Rust**, stable channel, via [rustup](https://rustup.rs).
- **Xcode Command Line Tools** — `xcode-select --install`.
- **CMake**, needed to build `whisper-rs`. `brew install cmake`.
- **Node.js 20+ and pnpm 9+** for the Tauri shell.

## Build and run

```bash
git clone https://github.com/cachevector/voxflow.git
cd voxflow
pnpm install
pnpm tauri dev
```

`pnpm tauri dev` runs the app with hot reload on the frontend. For a standalone build:

```bash
pnpm tauri build
```

The first Rust build is slow because the whole workspace and `whisper.cpp` compile from
scratch. Later builds are incremental.

## First run

1. Launch the app. It lives in the menu bar — there is no dock icon and no main window by
   default.
2. macOS will ask for **Microphone** access the first time VoxFlow captures audio.
3. Grant **Accessibility** access so the hotkey tap and paste can work. See
   [Permissions](./permissions) — nothing works without this one.
4. Open Settings from the tray icon and add a key for the rewrite pass, if you want one. See
   [Provider keys](./providers).

## Where things are stored

| What | Path |
|---|---|
| Settings | `~/Library/Application Support/com.maskedsyntax.VoxFlow/settings.json` |
| Whisper models | `~/Library/Application Support/com.maskedsyntax.VoxFlow/models/` |
| Provider keys | macOS Keychain (never in `settings.json`) |
| Transcript history | Local SQLite database in the same app support directory |

To remove VoxFlow completely, delete the app, that application support directory, and the
VoxFlow entries in Keychain Access.
