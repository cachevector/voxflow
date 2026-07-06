#!/usr/bin/env bash
# Install VoxFlow to /Applications for stable macOS privacy permissions.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/apps/macos/dist/VoxFlow.app"
DEST="/Applications/VoxFlow.app"

"$ROOT/scripts/build-macos.sh"

echo "Installing to $DEST …"
killall VoxFlow 2>/dev/null || true
rm -rf "$DEST"
cp -R "$SRC" "$DEST"

echo ""
echo "Installed. Launch with:"
echo "  open /Applications/VoxFlow.app"
echo ""
echo "Then enable permissions for /Applications/VoxFlow.app in System Settings."
