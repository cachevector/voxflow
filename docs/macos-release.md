# Shipping a signed macOS DMG

Users install VoxFlow by downloading `VoxFlow-macos-arm64.dmg` from
[GitHub Releases](https://github.com/cachevector/voxflow/releases/latest)
and dragging the app into Applications.

A Gatekeeper-clean open (no right-click workaround) needs a
**Developer ID Application** certificate and Apple notarization. This
machine currently has Apple Development and Apple Distribution identities
only. Those cannot notarize a direct-download DMG.

Team ID: `GVGJLY2H53`.

## One-time Apple setup

1. On [Certificates, Identifiers & Profiles](https://developer.apple.com/account/resources/certificates/list),
   create a **Developer ID Application** certificate. Only the account
   holder can create this type. Install the downloaded `.cer`.
2. Confirm it shows up:

   ```bash
   security find-identity -v -p codesigning
   ```

   You want a line like `Developer ID Application: Aftaab Siddiqui (GVGJLY2H53)`.
3. Create an [app-specific password](https://support.apple.com/en-ca/HT204397)
   for notarization, or an App Store Connect API key.

## Local build

```bash
export APPLE_ID="you@example.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="GVGJLY2H53"
./scripts/release-macos.sh
```

The script picks the Developer ID identity from the keychain when it
exists. The DMG lands at `target/release/bundle/dmg/VoxFlow-macos-arm64.dmg`.

## GitHub Actions

Push a `v*` tag, or run the **Release** workflow. To sign and notarize on
the runner, add these repository secrets:

| Secret | What it is |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64 of the exported Developer ID `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | Password for that `.p12` |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Aftaab Siddiqui (GVGJLY2H53)` |
| `APPLE_ID` | Apple ID email |
| `APPLE_PASSWORD` | App-specific password |
| `APPLE_TEAM_ID` | `GVGJLY2H53` |

Export the `.p12` from Keychain Access (My Certificates, export the
private key), then:

```bash
openssl base64 -A -in DeveloperID.p12 -out certificate-base64.txt
```

Never commit the `.p12`, the `.cer`, or `AuthKey_*.p8`.
