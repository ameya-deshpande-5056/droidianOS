# live-build

Build flow:

```sh
sh ./build/scripts/build-packages.sh
sh ./build/scripts/build-local-repo.sh
sh ./build/scripts/build-iso.sh
```

Local project packages are copied into `config/packages.chroot`.

The live image also includes the branded droidianOS installer hub and the first-login desktop chooser from `droidianos-session-defaults`.

The local repository is copied into:

```text
/opt/droidianos-repo
```

The installed system includes:

```text
/etc/apt/sources.list.d/droidianos-local.list
```

On first boot, the live-session autostart stack opens the branded installer hub on the live image and the desktop chooser on an installed system.

Prerequisite:

- `waydroid` must be available from an APT source during the live-build run.
- `curl`, `jq`, `sha256sum`, and `unzip` fetch and verify the latest official x86_64 Waydroid GAPPS system and MAINLINE vendor images.
