# droidianos-apk-installer

APK inspection backend.

Implemented command:

```text
droidianos-apk-inspect <file.apk|file.apks|file.apkm>
droidianos-apk-installer <file.apk|file.apks|file.apkm>
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

- registers APK, APKS, and APKM package formats
- installs `droidianos-apk-installer.desktop`
- sets the default handler for Android package files
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
- validates that split archives contain one base APK and matching package splits
- displays a confirmation dialog when launched through the UI
- stages APK files under Waydroid's user data directory
- installs single APKs with `waydroid app install`
- installs split APK sets atomically with Android `pm install-multiple` through a path-validating Polkit helper
- verifies the package through unprivileged `waydroid app list`
- refreshes app integration with `droidianos-integrationd --refresh`
- stores in-memory transaction status for the running service lifetime

Security metadata:

- installs Polkit action `org.droidianos.apk-installer.install` for split APK installation only

Flutter UI is not implemented yet.
