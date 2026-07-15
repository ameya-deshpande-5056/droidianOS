# Security Model

Default policy:

- No unrestricted home directory sharing with Android.
- Shared folders are explicit and stored in `~/.config/droidianos/shared-folders.json`.
- Shared folders outside the user's home directory are rejected by `apply-shared-folders`.
- APK installation stages files under `~/.cache/droidianos/apk-staging` before install.
- Permission state is stored in `~/.config/droidianos/permissions.json`.
- Runtime Android permissions are applied through `waydroid shell pm grant/revoke`.
- GUI tools do not run as root.

Current gaps:

- Shared-folder policy persistence exists, but full mount enforcement is still limited.
- APK provenance/update trust is not implemented.
- Notification replies/actions are not implemented.
