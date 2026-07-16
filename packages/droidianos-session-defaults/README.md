# droidianos-session-defaults

Desktop defaults, MIME registrations, and launcher policies.

Current policy:

- Hide direct Waydroid launcher entries from the desktop application menu.
- On the live ISO, show a branded installer hub first and explain the full setup flow before launching the underlying installer backend.
- After installation, show a first-login desktop chooser that installs the selected desktop environment, shows a screenshot preview, and can optionally install Flatpak support.
- The ISO includes Labwc as its lightweight Wayland base GUI; the live image boots into the installer hub, while the installed system drops into the desktop chooser on first login.
- Both screens use the shared droidianOS GTK visual style and the same branding language as the bootloader and wallpaper assets.
