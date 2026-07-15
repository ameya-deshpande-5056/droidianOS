# Security Threat Model

## Default Rules

- Android apps must not receive unrestricted home directory access.
- Shared folders must be explicitly configured.
- GUI applications must not run as root.
- Privileged operations must use polkit or existing system package tools.
- Runtime policy must live in XDG config/state paths.

## Current Controls

- APK install is mediated by `droidianos-apk-installerd`.
- App permissions are mediated by `droidianos-permissionsd`.
- Shared-folder policy is stored in `~/.config/droidianos/shared-folders.json`.
- Android runtime permissions use `waydroid shell pm grant/revoke`.

## Known Gaps

- Per-app shared-folder enforcement is not complete.
- Notification actions and replies are not complete.
- APK update provenance is not implemented.

