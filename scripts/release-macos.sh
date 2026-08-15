#!/usr/bin/env bash
# Build a macOS .dmg that people can download and drag into Applications.
#
# Signing, first match wins:
#   1. APPLE_SIGNING_IDENTITY if already set
#   2. Developer ID Application (needed to notarize a public DMG)
#   3. Apple Development (same as a local `pnpm tauri build` on this Mac)
#   4. ad-hoc, only if no certificate exists
# Notarization runs when APPLE_ID + APPLE_PASSWORD + APPLE_TEAM_ID (or an
# App Store Connect API key) are in the environment.
set -euo pipefail
cd "$(dirname "$0")/.."

pick_identity() {
  security find-identity -v -p codesigning | sed -n "s/.*\"\($1[^\"]*\)\".*/\1/p" | head -1
}

if [[ -z "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  if identity=$(pick_identity "Developer ID Application:"); [[ -n "${identity:-}" ]]; then
    export APPLE_SIGNING_IDENTITY="$identity"
  elif identity=$(pick_identity "Apple Development:"); [[ -n "${identity:-}" ]]; then
    export APPLE_SIGNING_IDENTITY="$identity"
  else
    export APPLE_SIGNING_IDENTITY="-"
  fi
fi

echo "signing with $APPLE_SIGNING_IDENTITY"

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
