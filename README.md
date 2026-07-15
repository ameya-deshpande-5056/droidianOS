# droidianOS

Debian Stable based desktop distribution with first-class Android app integration through Waydroid.

Current state: incremental implementation in progress.

## Build

On Debian Stable or a compatible build container:

```sh
sudo apt-get update
sudo apt-get install -y build-essential cargo devscripts debhelper dpkg-dev libdbus-1-dev rustc
sh ./build/scripts/build-packages.sh
```

Artifacts are written to `build/artifacts/packages`.

## ISO

On Debian Stable:

```sh
sudo apt-get install -y apt-utils build-essential cargo debhelper devscripts dpkg-dev libdbus-1-dev live-build rustc
sh ./build/scripts/build-packages.sh
sh ./build/scripts/build-local-repo.sh
sh ./build/scripts/build-iso.sh
```

ISO artifacts are written to `build/artifacts/iso`.

## Release Artifacts

After building and validating an ISO:

```sh
DROIDIANOS_GPG_KEY=<key-id> sh ./build/scripts/release-artifacts.sh
```

Release files are written to `build/artifacts/release`.

## GitHub Build

Pushing to GitHub runs `.github/workflows/build.yml`. Pull requests build packages only. Pushes and manual runs build packages, the local APT repository, the ISO, release artifacts, and the GHCR package.

Successful builds publish the generated APT repository as a GHCR package so it appears under the repository Packages page. Tagged builds create a GitHub Release containing ISO files, release metadata, and generated `.deb` package artifacts. Manual `workflow_dispatch` runs also publish a prerelease named `build-latest` with the same downloadable files.

Download the ISO from the repository's Releases page, not the Packages page:

- `https://github.com/<owner>/<repo>/releases`

Required repository secret:

- `WAYDROID_APT_SOURCE`: APT source line that provides the `waydroid` package for Debian Bookworm live-build.

Optional repository secrets:

- `WAYDROID_APT_KEY_ASC`: ASCII-armored signing key for the Waydroid APT source.
- `DROIDIANOS_GPG_PRIVATE_KEY`: ASCII-armored release signing private key.
- `DROIDIANOS_GPG_KEY`: GPG key ID used for repository and checksum signatures.
- `DROIDIANOS_GPG_PASSPHRASE`: passphrase for the release signing key, if set.

## License

MIT. See `LICENSE`.
