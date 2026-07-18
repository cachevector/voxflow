# Phase 0 spike results (2026-07-18)

Risk-spike pass before building out Phase 1 product code, per the
[GPUI rewrite plan](../voxflow_project_spec_performance_costs.txt). Run on
the user's real machine (Zorin OS 18.1, Intel + NVIDIA GPU), primarily on
an active **X11** session (`DISPLAY=:1`). GNOME Wayland-specific items were
deferred — see below.

GPUI pinned to `zed-industries/zed` git rev `952d712dac48a4af2c54fb22c82d82a9d69b72d4`
(tag `v1.11.3`), chosen because it postdates the `wlr-layer-shell` merge
([PR #35610](https://github.com/zed-industries/zed/pull/35610), merged
2025-10-29) and is a tagged stable release rather than an arbitrary `main`
commit.

## Go / no-go per risk item

| # | Item | Platform tested | Result |
|---|------|------------------|--------|
| 0.1 | `gpui`/`gpui_platform`/`gpui_tokio` git-pinned deps compile | X11 (build only) | **GO.** Clean build (~2-4 min cold). One benign future-incompat warning from a transitive dep (`proc-macro-error2`), not actionable. |
| 0.3 | `WindowKind::PopUp` transparent/undecorated/bottom-center overlay | X11 | **GO.** Confirmed via screenshot: renders correctly, positioned bottom-center, stacks above all other windows, no visible titlebar/decoration. |
| 0.3 | `WindowKind::LayerShell` on GNOME Wayland | *(not run — GNOME doesn't support layer-shell at all; this was never expected to work there)* | **N/A by design.** GNOME always uses the `PopUp` fallback path per the plan. |
| 0.3 (Wayland re-check) | `PopUp` fallback's always-on-top behavior under real GNOME Wayland | Deferred | **DEFERRED** by user 2026-07-18. X11 behavior confirmed; Wayland's stricter window-stacking rules mean this should still be spot-checked before considering the overlay production-ready, but is not blocking Phase 1 start. |
| 0.4 | GPUI survives with zero windows open (tray-only background mode) | X11 | **GO.** Empirically confirmed: `application().run()` does not exit when the run closure returns without opening a window; background executor tasks kept firing for the full test duration. No keep-alive hidden window workaround needed. |
| 0.5 | `ManualBindingHotkey` Unix socket + `voxflowctl trigger <action>` | X11 (socket mechanism is platform-agnostic) | **GO.** All four test events (`toggle`, `down`, `up`, `toggle`) delivered correctly and in order. Only the *compositor-side keybinding* (GNOME Custom Shortcut → `voxflowctl trigger toggle`) remains to be wired up and tested on a real GNOME session — the IPC path itself is proven. |
| 0.6 | `X11Hotkey` via `global-hotkey`/XGrabKey | X11 | **GO.** Registered Super+Shift+Space; real physical key presses produced 8 clean Pressed/Released event pairs with no visible side effects (no leakage to focused apps, no WM conflict). True push-to-talk is viable on X11 with no extra plumbing. |
| 0.7 | `WaylandPortalInserter` (RemoteDesktop-synthesized paste) on GNOME | Deferred | **DEFERRED** by user 2026-07-18 — no real GNOME Wayland session available this pass. **Decision: default to `CopyOnly`** (clipboard + "Copied — press Ctrl+V" overlay state) as GNOME Wayland's primary insertion path, exactly as the plan's own fallback design anticipated. `WaylandPortalInserter` can still be built and offered as an opt-in upgrade, but must not be assumed reliable until validated on a real session. |
| 0.8 | `X11Inserter` (arboard clipboard + enigo/XTest synthetic Ctrl+V) | X11 | **GO.** Marker string round-tripped correctly: set clipboard → synthetic Ctrl+V → user confirmed the text appeared in the focused field. Matches existing `ClipboardPasteInserter` behavior. |

## Other findings (not go/no-go, but worth carrying into Phase 1)

- **`gpui_tokio::init(cx)` must be called first thing inside the `run` closure.** GPUI's Linux backend uses `zbus` internally (portal/desktop-environment integration, e.g. theme detection) and a background worker thread panics with *"there is no reactor running, must be called from the context of a Tokio 1.x runtime"* if no Tokio runtime is active. Even with `gpui_tokio::init` called first, one such panic still fires very early (apparently during X11 client bootstrap, before the `run` closure's own code executes) — it appears to be **non-fatal** (the window still renders and functions correctly despite it), but is a known rough edge to watch for, not something fixable from the embedding app's side.
- **`enigo`'s X11 backend requires the `libxdo-dev` system package** at build time (provides `-lxdo` for the linker) — this is *not* part of Zed's own `script/linux` dependency list (Zed doesn't do synthetic input), so it must be added explicitly to VoxFlow's own Linux setup docs/scripts.
- **The actual GPUI application entry point is `gpui_platform::application()`**, from a separate `gpui_platform` crate (with `x11`/`wayland`/`font-kit` features), not `gpui::Application::new()` as initially assumed from web research — confirmed directly from the pinned commit's own `crates/gpui/examples/*.rs`.
- System build dependencies actually required beyond Zed's documented list, on Zorin OS 18.1 (Ubuntu-based): the full `script/linux` apt list, **plus `libxdo-dev`**.

## Net effect on the roadmap

Every X11-testable and platform-agnostic item passed cleanly on the first or
second attempt, with two small but real fixes required (`gpui_tokio::init`,
`libxdo-dev`) now folded into `voxflow-app`. The only genuinely open risk
left is Wayland-specific: portal-synthesized paste on GNOME. Per the user's
decision, Phase 1 proceeds with `CopyOnly` as GNOME Wayland's default
insertion behavior rather than blocking on that validation — this is not a
downgrade from the plan, it's the plan's own documented fallback being
promoted to "primary" pending real-session validation.
