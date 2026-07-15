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
find "$ISO_DIR" -maxdepth 1 -type f \( -name '*.iso' -o -name '*.hybrid.iso' \) -exec cp {} "$RELEASE_DIR"/ \;

: > "$CHECKSUM_FILE"
found_iso=0
for iso in "$RELEASE_DIR"/*.iso "$RELEASE_DIR"/*.hybrid.iso; do
  [ -f "$iso" ] || continue
  found_iso=1
  (cd "$RELEASE_DIR" && sha256sum "$(basename "$iso")") >> "$CHECKSUM_FILE"
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
  gpg --batch --yes --local-user "$DROIDIANOS_GPG_KEY" --armor --detach-sign "$CHECKSUM_FILE"
else
  echo "DROIDIANOS_GPG_KEY not set; checksums were not signed" >&2
fi

echo "$RELEASE_DIR"
