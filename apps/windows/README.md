# VoxFlow Windows Shell (Phase 4)

WinUI 3 shell consuming `voxflow_ffi` via C#/WinRT bindings generated from the same UniFFI crate.

## Planned components

- `VoxFlow.Windows` — WinUI 3 tray + floating bar
- WASAPI audio via shared Rust CPAL backend
- `RegisterHotKey` for push-to-talk
- UI Automation text insertion with SendInput paste fallback
- Windows Credential Manager for API keys

## Build (stub)

```powershell
cargo build -p voxflow-ffi --release
# Generate C# bindings:
cargo run --bin uniffi-bindgen -- generate --library target/release/voxflow_ffi.dll --language csharp --out-dir apps/windows/VoxFlowCore
```

## Status

Shell project scaffold — implement WinUI 3 app against generated bindings.
