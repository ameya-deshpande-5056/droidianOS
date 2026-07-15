#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
ARTIFACT_DIR="$ROOT_DIR/build/artifacts/packages"

mkdir -p "$ARTIFACT_DIR"

for package_dir in "$ROOT_DIR"/packages/*; do
  [ -d "$package_dir/debian" ] || continue
  package_name="$(basename "$package_dir")"
  echo "Building $package_name"
  chmod +x "$package_dir/debian/rules"
  find "$package_dir/debian" -maxdepth 1 -type f \( -name 'preinst' -o -name 'postinst' -o -name 'prerm' -o -name 'postrm' \) -exec chmod +x {} \;
  (
    cd "$package_dir"
    dpkg-buildpackage -us -uc -b
  )
  find "$ROOT_DIR/packages" -maxdepth 1 -type f \( -name '*.deb' -o -name '*.changes' -o -name '*.buildinfo' \) -exec mv {} "$ARTIFACT_DIR"/ \;
done
