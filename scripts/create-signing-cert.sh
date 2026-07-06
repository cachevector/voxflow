#!/usr/bin/env bash
# Create a stable self-signed code-signing certificate for VoxFlow.
# This makes macOS Privacy permissions (Input Monitoring, Accessibility)
# persist across rebuilds — ad-hoc signing changes the code hash every build,
# which resets TCC grants.
set -euo pipefail

CERT_NAME="VoxFlow Self-Signed"
KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"

if security find-certificate -c "$CERT_NAME" >/dev/null 2>&1; then
  echo "Signing certificate already exists: $CERT_NAME"
  exit 0
fi

echo "Creating self-signed code-signing certificate: $CERT_NAME"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

cat > "$TMP/cert.cfg" <<'CFG'
[ req ]
distinguished_name = dn
x509_extensions = v3
prompt = no
[ dn ]
CN = VoxFlow Self-Signed
[ v3 ]
basicConstraints = critical, CA:false
keyUsage = critical, digitalSignature
extendedKeyUsage = critical, codeSigning
CFG

openssl req -x509 -newkey rsa:2048 -nodes -days 3650 \
  -keyout "$TMP/key.pem" -out "$TMP/cert.pem" -config "$TMP/cert.cfg" >/dev/null 2>&1

openssl pkcs12 -export -out "$TMP/id.p12" \
  -inkey "$TMP/key.pem" -in "$TMP/cert.pem" \
  -name "$CERT_NAME" -passout pass:voxflow >/dev/null 2>&1

# -A: allow all apps (incl. codesign) to use the key without a keychain prompt.
security import "$TMP/id.p12" -k "$KEYCHAIN" -P voxflow -A -T /usr/bin/codesign >/dev/null 2>&1

# Trust the cert for code signing so codesign accepts it non-interactively.
security add-trusted-cert -d -r trustAsRoot -p codeSign \
  -k "$KEYCHAIN" "$TMP/cert.pem" >/dev/null 2>&1 || true

echo "Certificate created and imported into login keychain."
