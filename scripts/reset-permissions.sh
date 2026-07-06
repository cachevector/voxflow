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
echo "  1. open apps/macos/dist/VoxFlow.app"
echo "  2. System Settings → Privacy & Security"
echo "  3. Enable VoxFlow in Input Monitoring, Accessibility, and Microphone"
echo "  4. Menu bar → VoxFlow → Reconnect Hotkey"
