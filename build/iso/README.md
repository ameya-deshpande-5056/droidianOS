# ISO

Generated ISO artifacts will be placed under `build/artifacts/iso`.

Required host tools:

- `live-build`
- `dpkg-dev`
- `apt-utils`

Build command:

```sh
sh ./build/scripts/build-iso.sh
```

Current prerequisite:

- The `waydroid` package must be available to APT during the live-build run.

Release packaging:

```sh
DROIDIANOS_GPG_KEY=<key-id> sh ./build/scripts/release-artifacts.sh
```
