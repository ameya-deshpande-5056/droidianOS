# droidianos-software-center

Unified software center MVP.

Implemented command:

```text
droidianos-software-center
```

Current features:

- view installed APT, Flatpak, and Android applications in one list
- remove selected APT packages through `pkexec apt-get remove -y`
- remove selected Flatpak applications through `flatpak uninstall -y`
- remove selected Android apps through `waydroid app remove`
- search APT packages through `apt-cache search`
- install APT packages through `pkexec apt-get install -y`
- search Flatpak applications through `flatpak search`
- install Flatpak applications from `flathub` through `flatpak install -y flathub`
- update Flatpak applications through `flatpak update -y`
- select and install APK files through `droidianos-apk-installer`

Current UI:

- Zenity-based MVP

Flutter UI is not implemented yet.
