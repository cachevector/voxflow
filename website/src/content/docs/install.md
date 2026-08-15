---
title: Install on macOS
description: Download the VoxFlow DMG, drag it into Applications, and start dictating.
group: Start here
order: 2
sidebarLabel: Install
---

VoxFlow ships as a signed macOS disk image. You do not need Rust, Node, or a
compiler.

## Download and install

1. Download [VoxFlow for macOS](https://github.com/cachevector/voxflow/releases/latest/download/VoxFlow-macos-arm64.dmg)
   (Apple Silicon).
2. Open the `.dmg` and drag **VoxFlow** into **Applications**.
3. Eject the disk image and launch VoxFlow from Applications or Spotlight.
4. The app lives in the menu bar. There is no dock icon and no main window by
   default.

macOS 13 or later. Apple Silicon is what the release build targets, so Whisper
can use Metal.

If macOS says the app cannot be opened because it is from an unidentified
developer, right-click VoxFlow in Applications, choose **Open**, then confirm.
That prompt goes away once the release is notarized with a Developer ID.

## First run

1. macOS will ask for **Microphone** access the first time VoxFlow captures
   audio.
2. Grant **Accessibility** so the hotkey and paste can work. See
   [Permissions](./permissions). Nothing works without this one.
3. Open Settings from the tray icon and add a key for the rewrite pass, if you
   want one. See [Provider keys](./providers).
4. Focus any text field. Hold <kbd>⌥</kbd> <kbd>⌃</kbd> and speak.

## Where things are stored

| What | Path |
|---|---|
| Settings | `~/Library/Application Support/com.maskedsyntax.VoxFlow/settings.json` |
| Whisper models | `~/Library/Application Support/com.maskedsyntax.VoxFlow/models/` |
| Provider keys | macOS Keychain (never in `settings.json`) |
| Transcript history | Local SQLite database in the same application support directory |

To remove VoxFlow completely, delete the app, that application support
directory, and the VoxFlow entries in Keychain Access.

## Build from source

If you want a development build instead of the DMG:

```bash
git clone https://github.com/cachevector/voxflow.git
cd voxflow
pnpm install
pnpm tauri dev
```

You will need Rust (stable), Xcode Command Line Tools, CMake, Node.js 20+, and
pnpm 9+. Details are in [docs/macos-setup.md](https://github.com/cachevector/voxflow/blob/master/docs/macos-setup.md).
