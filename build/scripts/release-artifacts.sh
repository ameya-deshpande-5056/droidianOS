#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
ISO_DIR="$ROOT_DIR/build/artifacts/iso"
RELEASE_DIR="$ROOT_DIR/build/artifacts/release"
CHECKSUM_FILE="$RELEASE_DIR/SHA256SUMS"

if [ ! -d "$ISO_DIR" ]; then
  echo "ISO artifact directory not found: $ISO_DIR" >&2
  exit 1
fi

mkdir -p "$RELEASE_DIR"

: > "$CHECKSUM_FILE"
found_iso=0
for iso in "$ISO_DIR"/*.iso; do
  [ -f "$iso" ] || continue
  found_iso=1
  (cd "$ISO_DIR" && sha256sum "$(basename "$iso")") >> "$CHECKSUM_FILE"
done

if [ "$found_iso" -eq 0 ]; then
  echo "no ISO artifacts found" >&2
  exit 1
fi

sort "$CHECKSUM_FILE" -o "$CHECKSUM_FILE"

cp "$ROOT_DIR/docs/releases/1.0-release-notes.md" "$RELEASE_DIR/RELEASE_NOTES.md"
cp "$ROOT_DIR/docs/releases/1.0-security-notes.md" "$RELEASE_DIR/SECURITY_NOTES.md"
cp "$ROOT_DIR/docs/releases/1.0-upgrade-path.md" "$RELEASE_DIR/UPGRADE.md"
cp "$ROOT_DIR/docs/releases/1.0-known-issues.md" "$RELEASE_DIR/KNOWN_ISSUES.md"
cp "$ROOT_DIR/docs/releases/1.0-manifest.template.json" "$RELEASE_DIR/release-manifest.json"

if [ -n "${DROIDIANOS_GPG_KEY:-}" ]; then
  if [ -n "${DROIDIANOS_GPG_PASSPHRASE:-}" ]; then
    gpg --batch --yes --pinentry-mode loopback --passphrase "$DROIDIANOS_GPG_PASSPHRASE" --local-user "$DROIDIANOS_GPG_KEY" --armor --detach-sign "$CHECKSUM_FILE"
  else
    gpg --batch --yes --pinentry-mode loopback --local-user "$DROIDIANOS_GPG_KEY" --armor --detach-sign "$CHECKSUM_FILE"
  fi
else
  echo "DROIDIANOS_GPG_KEY not set; checksums were not signed" >&2
fi

echo "$RELEASE_DIR"
