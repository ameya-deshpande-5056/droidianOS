#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
PACKAGE_DIR="$ROOT_DIR/build/artifacts/packages"
REPO_DIR="$ROOT_DIR/build/artifacts/repo"
BINARY_DIR="$REPO_DIR/dists/stable/main/binary-amd64"
POOL_DIR="$REPO_DIR/pool/main"

if [ ! -d "$PACKAGE_DIR" ]; then
  echo "package artifact directory not found: $PACKAGE_DIR" >&2
  exit 1
fi

rm -rf "$REPO_DIR"
mkdir -p "$BINARY_DIR" "$POOL_DIR"

find "$PACKAGE_DIR" -maxdepth 1 -type f -name '*.deb' -exec cp {} "$POOL_DIR"/ \;

(
  cd "$REPO_DIR"
  dpkg-scanpackages --multiversion pool /dev/null > "$BINARY_DIR/Packages"
  gzip -kf "$BINARY_DIR/Packages"
  if command -v apt-ftparchive >/dev/null 2>&1; then
    apt-ftparchive release dists/stable > dists/stable/Release
  else
    cat > dists/stable/Release <<'EOF'
Origin: droidianOS
Label: droidianOS
Suite: stable
Codename: stable
Architectures: amd64
Components: main
Description: Local droidianOS build repository
EOF
  fi
  if [ -n "${DROIDIANOS_GPG_KEY:-}" ]; then
    gpg --batch --yes --local-user "$DROIDIANOS_GPG_KEY" --detach-sign --armor -o dists/stable/Release.gpg dists/stable/Release
    gpg --batch --yes --local-user "$DROIDIANOS_GPG_KEY" --clearsign -o dists/stable/InRelease dists/stable/Release
  fi
)

echo "$REPO_DIR"
