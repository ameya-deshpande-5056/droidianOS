# droidianos-integrationd

User-session integration daemon.

Implemented commands:

- `droidianos-integrationd --refresh`
- `droidianos-integrationd --list`
- `droidianos-integrationd --daemon`
- `droidianos-launch <package>`
- `droidianos-notification-bridge`
- `droidianos-profile`

Registry path:

```text
~/.local/share/droidianos/apps.json
```

Generated desktop entries:

```text
~/.local/share/applications/droidianos-*.desktop
```

Generated fallback icons:

```text
~/.local/share/icons/hicolor/scalable/apps/droidianos-*.svg
```

Current discovery source:

```text
waydroid shell cmd package list packages -3
```

Current launch command:

```text
waydroid app launch <package>
```

D-Bus methods implemented:

- `org.droidianos.Integration.ListApps`
- `org.droidianos.Integration.RefreshApps`
- `org.droidianos.Integration.LaunchApp`

D-Bus signals emitted:

- `org.droidianos.Integration.AppStateChanged`

D-Bus interface definitions exist in `shared/dbus-interfaces`.

Real APK resource icon extraction is not implemented yet. This increment generates deterministic fallback SVG icons so desktop entries resolve to app-specific icon names.

Notification bridge:

- polls `waydroid shell cmd notification list`
- emits Linux notifications with `notify-send`
- suppresses duplicate notification keys in memory

Notification action buttons and inline replies are not implemented yet.

Performance profile:

```sh
droidianos-profile ~/droidianos-profile.txt
```

The package installs a systemd user service and preset:

```text
droidianos-integrationd.service
90-droidianos-integrationd.preset
```
