# VoxFlow Ground-Up Rewrite: Rust + Tauri + React (Windows + macOS)

> This is the architecture/planning document produced before the rewrite began. It's kept here (rather than only in a local planning tool) so the project is fully self-contained and can be picked up from any machine.

> **Pending change, not yet executed:** the plan below still targets Windows + macOS. There's an approved-but-unexecuted plan to drop Windows and target Linux (Wayland-first, X11 secondary) instead — see `docs/LINUX_MIGRATION_PLAN.md`. Do that swap before starting any new Windows-specific work.

## Status (as of 2026-07-16)

**Phase 0 is complete and verified working.** What exists right now:

- Old code deleted: Linux app, SwiftUI macOS app, UniFFI crate (`voxflow-ffi`), Flatpak packaging, Swift/UniFFI-specific scripts.
- Rust workspace rebuilt around the new architecture — see "Rust core architecture" below. New crates `voxflow-provider` (generic OpenAI-compatible client) and `voxflow-secrets` (OS keychain wrapper) exist and are wired into `voxflow-core`'s pipeline. `voxflow-config`, `voxflow-router`, `voxflow-insert` were all updated per the plan.
- Real Tauri v2 + React app exists at `src-tauri/` + `src/`: two windows (hidden settings window, floating transparent overlay), tray icon, global-shortcut hotkey registration (with graceful handling if the combo is already taken by another app), 10 settings pages, and an animated waveform overlay component.
- Text insertion is currently **clipboard-paste only** (`crates/voxflow-insert/src/clipboard_paste.rs`, cross-platform via `arboard` + `enigo`) — this was a deliberate Phase 0 scope decision (see Phase 0 bullet below); real Accessibility (macOS) / UI Automation (Windows) insertion is Phase 1 work, not yet started.
- Verified end-to-end: `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings`, and `cargo test --workspace` all pass clean. The frontend type-checks (`tsc --noEmit`) and builds (`vite build`). The actual compiled `voxflow.exe` was run on a real Windows machine — it launched, registered the tray icon and hotkey, and stayed running with no crash.
- CI (`.github/workflows/ci.yml`) already targets Windows + macOS only, with a separate frontend typecheck job.

**Not done yet (Phase 1, see roadmap below):**
- VAD is implemented as a crate (`voxflow-vad`) but not yet exercised with a real end-to-end recording → transcribe round trip against a live OpenAI key.
- No live waveform data yet — `events::emit_amplitude` exists in `src-tauri/src/events.rs` but nothing calls it; the audio capture thread doesn't stream RMS samples out yet. The overlay UI is ready to receive them (`dictation://amplitude` event) as soon as it's wired up.
- Real Accessibility/UI Automation text insertion (currently clipboard-paste only).
- No real end-to-end test against a self-hosted Custom Endpoint (e.g. a Raspberry Pi) yet — the generic adapter (`voxflow-provider`) supports it by design (any base URL), but it hasn't been exercised against a real self-hosted server.
- Real hotkey rebinding UI (Settings → Hotkey currently just displays the current binding).
- Active-app detection is stubbed to `None` in `src-tauri/src/hotkey.rs` — per-app profiles and the sensitive-app blocklist won't trigger until this is wired up.
- Placeholder app icons only (`src-tauri/icons/*` are solid-color squares generated programmatically, not real artwork) — fine for dev builds, needs real icon design before any public distribution.

## Context

VoxFlow is meant to replace Wispr Flow: a global-hotkey voice dictation tool that transcribes speech and then runs a second AI pass to rewrite it into a clean, well-formed sentence before inserting it into whatever app currently has focus. The user is paying Wispr Flow $400/month and wants to build their own BYOK version, targeting well under $300/month in their own personal API usage.

The previous implementation was a real, partially-working prototype: a Rust workspace (9 crates) implementing a genuine end-to-end pipeline (CPAL audio capture → WebRTC VAD → provider routing → OpenAI transcription → rule-based cleanup → text insertion → SQLite history → cost tracking) with a mature SwiftUI/AppKit macOS app on top, talking to the Rust core via UniFFI. There was also a minimal Linux CLI app and a Windows README-only stub. None of it used Tauri or React.

There was also a real gap between promise and implementation: the docs and spec both claimed API keys lived in the OS keychain/credential manager, but the actual code stored them in a plaintext JSON settings file. This rewrite fixes that.

Key decisions made during planning:

1. **Pure BYOK, no hosted backend.** The app calls providers directly with whatever key/endpoint the user configures. There is no multi-tenant server to design — the $300/month target is just the user's own personal transcription+rewrite usage bill.
2. **Keep and adapt the existing spec** (`voxflow_project_spec_performance_costs.txt`) rather than starting the product spec from zero.
3. **Cloud-only transcription for MVP** — no embedded/in-process local Whisper (whisper.cpp/whisper-rs bindings). That's an explicit fast-follow.
4. **A generic "Custom Endpoint" provider type is in scope for MVP.** Point VoxFlow at a Raspberry Pi (or any machine) running a self-hosted OpenAI-API-compatible server (e.g. whisper.cpp-server + llama.cpp-server/Ollama), reachable via Tailscale, to drive marginal cost toward $0. One generic HTTP adapter, not provider-specific branches.
5. **The AI rewrite/cleanup pass is on by default for every transcript**, not Pro-gated, using the same provider/key as transcription by default but independently configurable.
6. **Secrets must use real OS-native secure storage** (Windows Credential Manager / macOS Keychain via the `keyring` crate).

## Repo restructure

**Deleted:** `apps/linux-cosmic/`, `apps/macos/` (SwiftUI/AppKit + UniFFI Swift bindings), `crates/voxflow-ffi/`, `packaging/flatpak/`, `docs/linux-wayland.md`, `apps/windows/README.md`, `scripts/build-macos.sh`/`create-signing-cert.sh`/`generate-bindings.sh`/`install-macos.sh`, the Linux CI job, `crates/voxflow-transcribe/` (superseded by `voxflow-provider`).

**Kept and adapted:** `voxflow_project_spec_performance_costs.txt` (platform/tech-stack/roadmap sections rewritten for Tauri+React, Windows/macOS-only, Custom Endpoint), `docs/architecture.md`/`performance-budget.md`/`provider-keys.md` (updated in place), the internal logic of the existing crates (audio/VAD/history/cost math carried forward unchanged).

**New:** `src-tauri/` (Tauri v2 app), `src/` (React+TS+Vite frontend), `crates/voxflow-provider/`, `crates/voxflow-secrets/`.

## Rust core architecture

- **`voxflow-config`**: `Settings` keeps `QualityMode`, `DictationMode`, `HotkeyConfig`, `CostControlConfig`, `PrivacyConfig`, `AppProfile`/`OutputMode`, `Snippet`, `DictionaryEntry`, `BarPosition`. Plaintext `openai_api_key` replaced with two independent `ProviderConfig` entries (`transcription_provider`, `rewrite_provider`), each `{ kind: OpenAi | Groq | CustomEndpoint, base_url, model, accurate_model, api_key_ref }` where `api_key_ref` is a keyring lookup key, never the raw secret. `rewrite_enabled: bool` defaults `true`. `LicenseInfo` kept as a dormant field for later monetization but no longer gates the hot path.
- **`voxflow-secrets`**: wraps `keyring` with `set_secret`/`get_secret`/`delete_secret` keyed by stable IDs (`"transcription"`, `"rewrite"`).
- **`voxflow-provider`**: one `OpenAiCompatibleClient { base_url, api_key }` with `transcribe()` (multipart POST `{base_url}/audio/transcriptions`) and `chat_rewrite()` (POST `{base_url}/chat/completions`) — works unmodified against OpenAI, Groq, or a self-hosted server. Also has `rewrite::apply_rules`/`apply_snippets`/`apply_dictionary`/`system_prompt_for_mode` (rule-based cleanup + per-`OutputMode` system prompt construction).
- **`voxflow-router`**: `ProviderRouter::decide` returns a `ModelTier` (`Cheap`/`Accurate`) instead of a provider target — routes by quality mode, duration, noise, cost cap, sensitive-app blocklist, and per-app `disable_cloud`.
- **`voxflow-insert`**: `TextInserter` trait unchanged; `ClipboardPasteInserter` (new) is the cross-platform clipboard+simulated-paste implementation used for Phase 0/1. Real Accessibility/UI Automation implementations are Phase 1 additions layered in front of this fallback.
- **`voxflow-history`, `voxflow-cost`**: unchanged, provider-agnostic. `UsageRecord.was_local` renamed to `was_self_hosted` (true when `ProviderKind::CustomEndpoint`).
- **`voxflow-core`**: `DictationPipeline` builds two `OpenAiCompatibleClient`s per dictation (transcription + rewrite) via `resolve_client()`, which looks up the secret from `voxflow-secrets` by `api_key_ref`. Rewrite gate is `if settings.rewrite_enabled` (no more Pro check). `DictationEngine` (in `engine.rs`) exposes the sync API consumed by Tauri commands: `get_settings`, `save_settings`, `cost_dashboard`, `list_history`, `export_history_json/csv`, `on_hotkey_down`, `on_hotkey_up`.

## Tauri app structure

Two windows in `src-tauri/tauri.conf.json`: `main` (hidden settings window, shown from the tray) and `overlay` (transparent/undecorated/always-on-top pill, shown/hidden imperatively from Rust in `src-tauri/src/windows.rs`, positioned via `tauri-plugin-positioner`).

Plugins registered in `src-tauri/src/lib.rs`: `global-shortcut`, `autostart`, `single-instance`, `positioner`, `clipboard-manager`, `notification`, `store`. Tray icon + menu in `src-tauri/src/tray.rs`. Hotkey press/release handling in `src-tauri/src/hotkey.rs` calls `DictationEngine` directly and emits `dictation://state` events (see `src-tauri/src/events.rs`) that the React overlay listens to.

Commands live in `src-tauri/src/commands/` (one file per concern: `settings`, `secrets`, `cost`, `history`, `audio`), registered in `lib.rs`'s `invoke_handler!`.

## React frontend structure

`src/settings/` — sidebar-navigated settings app (`App.tsx` + `pages/`: General, Hotkey, Microphone, Transcription, AIRewrite, Providers, CostControl, PerAppProfiles, Privacy, Advanced), backed by `useSettings.ts` (loads/patches/saves via `@/shared/tauri.ts`'s typed `commands` object). `ProviderConfigEditor.tsx` is shared between the Transcription and AIRewrite pages.

`src/overlay/` — the floating pill (`App.tsx` + `components/`: `Waveform.tsx` canvas-rendered amplitude bars, `StateLabel.tsx`, `Timer.tsx`), animated with `framer-motion`, styled with Tailwind per the spec's visual language (rounded pill, subtle blur, no shiny gradients).

`src/shared/types.ts` mirrors the Rust `Settings`/`StateEvent`/`CostDashboard`/etc. structs by hand — keep these in sync manually when Rust types change.

## Platform permissions & integration

**macOS**: Microphone (auto-prompts via CPAL/AVFoundation), Accessibility (needed once real direct text insertion lands — Phase 1), Input Monitoring (only if a bare-modifier hotkey mode is added later; the current default is a combo binding that doesn't need it).

**Windows**: no formal mic permission prompt, but the Settings → Privacy → Microphone toggle can block CPAL — surface failures clearly. UI Automation insertion (Phase 1) needs no special entitlement. Unsigned dev builds trigger SmartScreen friction — expected until a code-signing cert is set up.

**Code signing**: macOS Developer ID + notarization ($99/yr), Windows Authenticode cert (~$100-400/yr) — needed before any public distribution, not for local dev.

## Build/tooling & CI

`.github/workflows/ci.yml`: `rust` job matrix `[windows-latest, macos-latest]` (fmt/clippy/test), `frontend` job on `ubuntu-latest` (tsc typecheck — fine, it's just where the JS tooling runs), `tauri-build` job matrix (unsigned debug builds). `scripts/dev.sh`/`.ps1` and `scripts/build.sh`/`.ps1` wrap `pnpm tauri dev`/`build`. `scripts/reset-permissions.sh` still useful for macOS TCC resets during dev.

## Phased roadmap

- **Phase 0 — Prototype.** ✅ Done (see Status above).
- **Phase 1 — Private MVP.** VAD wired into a real recording loop; rewrite pass exercised against a real key; Custom Endpoint validated against a real self-hosted server; cost dashboard/caps live-tested; real platform text insertion (Accessibility/UIA); permission onboarding UI; waveform visualization actually receiving amplitude data; error handling for empty audio/network/auth failures; active-app detection wired up.
- **Phase 2 — Public-facing polish.** Per-app intelligence profiles exercised for real; snippets + dictionary; better onboarding; crash-reporting/analytics opt-in; code signing.
- **Phase 3 — "V1 Pro" feature set.** Custom rewrite commands; saved provider presets; expanded per-app coverage.

No phase includes Linux, embedded local Whisper inference, or a hosted multi-tenant backend.

## Picking this up on a fresh machine

1. `git clone`, then `pnpm install` (Node 20+, pnpm via `corepack enable pnpm`).
2. `cargo build` from the repo root pulls the whole Rust workspace (needs Rust stable + the Tauri platform prerequisites — see `docs/macos-setup.md` / `docs/windows-setup.md`).
3. `pnpm tauri dev` (or `scripts/dev.sh` / `scripts/dev.ps1`) for a live dev loop.
4. To actually dictate anything, add a real provider key in Settings → Providers (stored in the OS keychain, not in the repo) — nothing works end-to-end without one, by design.
5. Start Phase 1 with VAD + real end-to-end transcription, per the roadmap above.
