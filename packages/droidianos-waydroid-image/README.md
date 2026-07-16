# droidianos-waydroid-image

Initializes embedded Waydroid images during OS installation and verifies the container on first boot.

Installed components:

- `/usr/lib/droidianos/waydroid-install`
- `droidianos-waydroid-firstboot.service`
- `/usr/lib/droidianos/waydroid-firstboot`
- `/usr/bin/droidianos-arm-setup`
- `/usr/lib/droidianos/install-arm-translation`
- `droidianos-shared-folders.service`
- `/usr/lib/droidianos/apply-shared-folders`

The Debian Installer initializes the embedded GAPPS system and MAINLINE vendor images. First boot starts and verifies `waydroid-container.service`, then writes `/var/lib/droidianos/waydroid-firstboot-complete`.

The GAPPS image contains Google Play services and the Google Play Store. Google may initially report the Waydroid device as uncertified; users must complete Google's device-registration process when required. At first login, the ARM setup asks for explicit consent before downloading proprietary community translation files directly to the installed machine. Intel systems use Houdini; other x86-64 systems use libndk_translation. Both paths configure and verify ARM32 and ARM64 ABIs. Downloads and installer source are revision-pinned and SHA-256 verified. The current payloads support Android 13; setup does not write ready state if the embedded image uses another Android release.

Shared-folder policy:

```text
~/.config/droidianos/shared-folders.json
```

The helper rejects shared folders outside the user's home directory.
