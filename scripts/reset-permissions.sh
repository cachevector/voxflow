#!/usr/bin/env bash
# Reset macOS privacy permissions for VoxFlow after a rebuild.
# Run this, then launch VoxFlow and re-enable toggles in System Settings.
set -euo pipefail

BUNDLE_ID="com.maskedsyntax.VoxFlow"

echo "Resetting TCC permissions for $BUNDLE_ID …"
tccutil reset Accessibility "$BUNDLE_ID" 2>/dev/null || true
tccutil reset ListenEvent "$BUNDLE_ID" 2>/dev/null || true
tccutil reset Microphone "$BUNDLE_ID" 2>/dev/null || true

echo ""
echo "Done. Now:"
echo "  1. pnpm tauri dev   (or open the built .app)"
echo "  2. System Settings → Privacy & Security"
echo "  3. Enable VoxFlow in Accessibility and Microphone"
echo "     (Input Monitoring only needed if you use the advanced bare-modifier hotkey)"
