#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
LIVE_BUILD_DIR="$ROOT_DIR/build/live-build"
ARTIFACT_DIR="$ROOT_DIR/build/artifacts/iso"
REPO_DIR="$ROOT_DIR/build/artifacts/repo"
PACKAGE_DIR="$ROOT_DIR/build/artifacts/packages"

if ! command -v lb >/dev/null 2>&1; then
  echo "live-build is required" >&2
  exit 1
fi

if [ ! -d "$REPO_DIR" ]; then
  "$ROOT_DIR/build/scripts/build-local-repo.sh" >/dev/null
fi

sh "$ROOT_DIR/build/scripts/fetch-waydroid-images.sh"

mkdir -p "$ARTIFACT_DIR"
rm -f "$ARTIFACT_DIR"/*.iso
rm -rf "$LIVE_BUILD_DIR/config/includes.chroot/opt/droidianos-repo"
rm -rf "$LIVE_BUILD_DIR/config/packages.chroot"
rm -rf "$LIVE_BUILD_DIR/config/includes.binary/isolinux"
mkdir -p "$LIVE_BUILD_DIR/config/includes.chroot/opt"
mkdir -p "$LIVE_BUILD_DIR/config/includes.binary/isolinux"
mkdir -p "$LIVE_BUILD_DIR/config/packages.chroot"
cp -a "$REPO_DIR" "$LIVE_BUILD_DIR/config/includes.chroot/opt/droidianos-repo"
cp "$ROOT_DIR/packages/droidianos-branding/usr/share/droidianos-branding/droidianos-bootloader.png" "$LIVE_BUILD_DIR/config/includes.binary/isolinux/splash.png"
find "$PACKAGE_DIR" -maxdepth 1 -type f -name '*.deb' -exec cp {} "$LIVE_BUILD_DIR/config/packages.chroot"/ \;

(
  cd "$LIVE_BUILD_DIR"
  chmod +x auto/config config/hooks/normal/010-droidianos-repo.hook.chroot config/hooks/normal/900-droidianos-boot-menu.hook.binary
  lb clean
  lb build
)

run_number="${GITHUB_RUN_NUMBER:-local}"
timestamp="$(date -u +%Y%m%d_%H%M%S)"
find "$LIVE_BUILD_DIR" -maxdepth 1 -type f -name '*.iso' -exec mv -f {} "$ARTIFACT_DIR/droidianOS-alpha_${run_number}_${timestamp}.iso" \;

echo "$ARTIFACT_DIR"
