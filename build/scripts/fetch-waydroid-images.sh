#!/bin/sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
IMAGE_DIR="$ROOT_DIR/build/live-build/config/includes.chroot/etc/waydroid-extra/images"
CACHE_DIR="$ROOT_DIR/build/cache/waydroid"

mkdir -p "$IMAGE_DIR" "$CACHE_DIR"

fetch_image() {
  manifest_url="$1"
  image_name="$2"
  latest="$(curl --fail --silent --show-error --location "$manifest_url" | jq -cer '.response | max_by(.datetime)')"
  filename="$(printf '%s' "$latest" | jq -er '.filename')"
  download_url="$(printf '%s' "$latest" | jq -er '.url')"
  expected_sha256="$(printf '%s' "$latest" | jq -er '.id')"

  case "$filename" in
    */*|'' )
      echo "Invalid Waydroid image filename: $filename" >&2
      exit 1
      ;;
  esac

  archive="$CACHE_DIR/$filename"
  if [ ! -f "$archive" ] || [ "$(sha256sum "$archive" | awk '{print $1}')" != "$expected_sha256" ]; then
    rm -f "$archive.part"
    curl --fail --show-error --location --retry 3 --output "$archive.part" "$download_url"
    mv "$archive.part" "$archive"
  fi

  actual_sha256="$(sha256sum "$archive" | awk '{print $1}')"
  if [ "$actual_sha256" != "$expected_sha256" ]; then
    echo "Waydroid image checksum mismatch: $filename" >&2
    exit 1
  fi

  archive_entry="$(unzip -Z1 "$archive" | awk -v image="$image_name" '$0 == image || $0 ~ ("/" image "$") { print; exit }')"
  if [ -z "$archive_entry" ]; then
    echo "$image_name is missing from $filename" >&2
    exit 1
  fi

  unzip -p "$archive" "$archive_entry" > "$IMAGE_DIR/$image_name.part"
  if [ ! -s "$IMAGE_DIR/$image_name.part" ]; then
    echo "Extracted Waydroid image is empty: $image_name" >&2
    exit 1
  fi
  mv "$IMAGE_DIR/$image_name.part" "$IMAGE_DIR/$image_name"
  echo "Embedded $filename"
}

fetch_image "https://ota.waydro.id/system/lineage/waydroid_x86_64/GAPPS.json" system.img
fetch_image "https://ota.waydro.id/vendor/waydroid_x86_64/MAINLINE.json" vendor.img
