# droidianos-settings

Settings and permissions services.

Implemented service:

```text
droidianos-permissionsd
droidianos-settings
droidianos-diagnostics
droidianos-recovery
```

D-Bus methods:

- `org.droidianos.Permissions.ListPermissions`
- `org.droidianos.Permissions.SetPermission`

D-Bus signal:

- `org.droidianos.Permissions.PermissionsChanged`

Policy path:

```text
~/.config/droidianos/permissions.json
```

Supported states:

- `allowed`
- `denied`
- `default`

Runtime Android permission application:

- `waydroid shell pm grant <package> <permission>`
- `waydroid shell pm revoke <package> <permission>`

Settings MVP pages:

- application permissions
- storage and shared folder policy
- diagnostics

Shared folder policy path:

```text
~/.config/droidianos/shared-folders.json
```

Current UI:

- Zenity-based MVP

Flutter UI is not implemented yet.

Recovery:

```sh
droidianos-recovery status
droidianos-recovery restart
droidianos-recovery refresh
droidianos-recovery reset-waydroid
droidianos-recovery clear-cache
```

Diagnostics export:

```sh
droidianos-diagnostics ~/droidianos-diagnostics.tar.gz
```
