# droidianos-apk-installer

APK inspection backend.

Implemented command:

```text
droidianos-apk-inspect <file.apk>
droidianos-apk-installer <file.apk>
```

Implemented service:

```text
droidianos-apk-installerd
```

D-Bus methods:

- `org.droidianos.Installer.InspectApk`
- `org.droidianos.Installer.InstallApk`
- `org.droidianos.Installer.GetTransaction`

D-Bus signal:

- `org.droidianos.Installer.InstallProgress`

Desktop integration:

- registers `application/vnd.android.package-archive`
- installs `droidianos-apk-installer.desktop`
- sets the default handler for `.apk` files
- uses Zenity for the current confirmation UI

Current output:

- package name
- version name
- version code
- literal app label when present in `AndroidManifest.xml`
- APK size
- requested permissions
- permission class: `normal`, `sensitive`, `background`, or `unknown`

Current parser scope:

- extracts `AndroidManifest.xml` with `unzip -p`
- parses Android binary XML without external Rust crates
- does not resolve `@string/...` resources yet
- does not extract APK icons yet

Current install flow:

- validates APK metadata
- displays a confirmation dialog when launched through the UI
- stages APK files under `~/.cache/droidianos/apk-staging`
- installs with `waydroid app install <file.apk>`
- refreshes app integration with `droidianos-integrationd --refresh`
- stores in-memory transaction status for the running service lifetime

Security metadata:

- installs polkit action `org.droidianos.apk-installer.install`

Flutter UI is not implemented yet.
