#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo build -p voxflow-ffi
cargo run --bin uniffi-bindgen -- generate \
  --library "$ROOT/target/debug/libvoxflow_ffi.dylib" \
  --language swift \
  --out-dir apps/macos/VoxFlowCore
cp apps/macos/VoxFlowCore/voxflow_ffiFFI.h apps/macos/VoxFlowFFI/voxflow_ffiFFI.h
echo "Swift bindings written to apps/macos/VoxFlowCore"
