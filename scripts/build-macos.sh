#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "Building Rust core…"
cargo build -p voxflow-ffi --release

echo "Generating Swift bindings…"
cargo run --bin uniffi-bindgen -- generate \
  --library "$ROOT/target/release/libvoxflow_ffi.dylib" \
  --language swift \
  --out-dir apps/macos/VoxFlowCore

# SwiftPM systemLibrary target needs the C header beside module.modulemap
cp apps/macos/VoxFlowCore/voxflow_ffiFFI.h apps/macos/VoxFlowFFI/voxflow_ffiFFI.h

echo "Building macOS app…"
cd apps/macos
swift build -c release

APP_NAME="VoxFlow"
APP_DIR="$ROOT/apps/macos/dist/${APP_NAME}.app"
CONTENTS="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS/MacOS"

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR"
cp VoxFlow/Info.plist "$CONTENTS/Info.plist"
cp .build/release/VoxFlow "$MACOS_DIR/VoxFlow"
cp "$ROOT/target/release/libvoxflow_ffi.dylib" "$MACOS_DIR/"
chmod +x "$MACOS_DIR/VoxFlow"

# SwiftPM links against the source-tree dylib; repoint to the bundled copy inside the app.
for OLD_PATH in \
  "$ROOT/target/release/deps/libvoxflow_ffi.dylib" \
  "$ROOT/target/release/libvoxflow_ffi.dylib"; do
  install_name_tool -change "$OLD_PATH" "@loader_path/libvoxflow_ffi.dylib" "$MACOS_DIR/VoxFlow" 2>/dev/null || true
done

# Use a stable self-signed identity so Privacy permissions persist across rebuilds.
# Falls back to ad-hoc if the certificate is unavailable.
CERT_NAME="VoxFlow Self-Signed"
"$ROOT/scripts/create-signing-cert.sh" || true
if security find-certificate -c "$CERT_NAME" >/dev/null 2>&1; then
  SIGN_ID="$CERT_NAME"
  echo "Signing with stable identity: $SIGN_ID"
else
  SIGN_ID="-"
  echo "Signing ad-hoc (permissions may reset each rebuild)"
fi

ENTITLEMENTS="$ROOT/apps/macos/VoxFlow/VoxFlow.entitlements"
codesign --force --sign "$SIGN_ID" --entitlements "$ENTITLEMENTS" \
  "$MACOS_DIR/libvoxflow_ffi.dylib"
codesign --force --sign "$SIGN_ID" --entitlements "$ENTITLEMENTS" \
  --identifier com.maskedsyntax.VoxFlow "$MACOS_DIR/VoxFlow"
codesign --force --deep --sign "$SIGN_ID" --entitlements "$ENTITLEMENTS" \
  --identifier com.maskedsyntax.VoxFlow "$APP_DIR"
codesign --verify --verbose "$APP_DIR" 2>&1 | head -5

echo ""
echo "Done. Launch VoxFlow:"
echo "  open \"$APP_DIR\""
echo ""
echo "Or from Finder: apps/macos/dist/VoxFlow.app"
