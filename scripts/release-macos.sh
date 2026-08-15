#!/usr/bin/env bash
# Build a macOS .dmg that people can download and drag into Applications.
#
# Signing:
#   If a Developer ID Application certificate is in the keychain (or
#   APPLE_SIGNING_IDENTITY is already set), the app is signed with it.
#   Notarization runs automatically when APPLE_ID + APPLE_PASSWORD +
#   APPLE_TEAM_ID (or an App Store Connect API key) are in the environment.
#   Otherwise the DMG is ad-hoc signed: it installs, but other Macs need
#   right-click -> Open the first time.
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ -z "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  if identity=$(security find-identity -v -p codesigning | sed -n 's/.*"\(Developer ID Application:[^"]*\)".*/\1/p' | head -1); then
    if [[ -n "$identity" ]]; then
      export APPLE_SIGNING_IDENTITY="$identity"
      echo "signing with $APPLE_SIGNING_IDENTITY"
    fi
  fi
fi

if [[ -z "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  export APPLE_SIGNING_IDENTITY="-"
  echo "no Developer ID Application certificate found; building an ad-hoc signed DMG"
  echo "other Macs will need right-click -> Open the first time until a Developer ID is installed"
fi

if [[ -n "${APPLE_ID:-}" && -n "${APPLE_PASSWORD:-}" && -n "${APPLE_TEAM_ID:-}" ]]; then
  echo "notarization credentials present; Tauri will submit the app after signing"
elif [[ -n "${APPLE_API_KEY:-}" && -n "${APPLE_API_ISSUER:-}" && -n "${APPLE_API_KEY_PATH:-}" ]]; then
  echo "App Store Connect API key present; Tauri will submit the app after signing"
else
  echo "no notarization credentials in the environment; skipping notarization"
fi

pnpm install
pnpm tauri build --bundles dmg,app

dmg_dir="target/release/bundle/dmg"
shopt -s nullglob
dmgs=("$dmg_dir"/VoxFlow_*.dmg)
shopt -u nullglob
if (( ${#dmgs[@]} == 0 )); then
  echo "no VoxFlow_*.dmg produced under $dmg_dir" >&2
  exit 1
fi

versioned="${dmgs[0]}"
stable="$dmg_dir/VoxFlow-macos-arm64.dmg"
cp -f "$versioned" "$stable"

echo
echo "DMG ready:"
echo "  $versioned"
echo "  $stable"
shasum -a 256 "$stable"
echo
echo "Publish with:"
echo "  gh release create v0.1.0 --title \"VoxFlow 0.1.0\" --notes-file - \"$stable#VoxFlow-macos-arm64.dmg\" \"$versioned\""
