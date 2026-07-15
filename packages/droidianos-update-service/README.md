# droidianos-update-service

Unified update service.

Implemented service:

```text
droidianos-updated
```

D-Bus methods:

- `org.droidianos.Updates.CheckUpdates`
- `org.droidianos.Updates.ListUpdates`
- `org.droidianos.Updates.ApplyUpdates`

D-Bus signals:

- `org.droidianos.Updates.UpdatesChanged`
- `org.droidianos.Updates.UpdateProgress`

Adapters:

- APT check: `apt list --upgradable`
- APT apply: `pkexec apt-get upgrade -y`
- Flatpak check: `flatpak remote-ls --updates`
- Flatpak apply: `flatpak update -y`
- Android image apply: `waydroid upgrade`

Current limitation:

- `ApplyUpdates` applies all supported updates. Per-update ID filtering is not implemented yet.
- Android image update availability check is not implemented yet.
