# droidianos-waydroid-image

Initializes Waydroid on first boot.

Installed components:

- `droidianos-waydroid-firstboot.service`
- `/usr/lib/droidianos/waydroid-firstboot`
- `droidianos-shared-folders.service`
- `/usr/lib/droidianos/apply-shared-folders`

The service runs once, calls `waydroid init` when images are missing, starts `waydroid-container.service` when available, then writes `/var/lib/droidianos/waydroid-initialized`.

microG image customization is not implemented in this increment.

Shared-folder policy:

```text
~/.config/droidianos/shared-folders.json
```

The helper rejects shared folders outside the user's home directory.
