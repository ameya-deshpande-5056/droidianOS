# droidianOS

droidianOS is a Debian Stable based desktop distribution that integrates Android applications into the Linux desktop through Waydroid.

Current status: alpha and under active development.

## Implemented

- Builds against the Debian release identified as Stable at ISO build time.
- Uses a lightweight Labwc/Wayland base session with a first-login desktop chooser.
- Offers GNOME, KDE Plasma, Xfce, Cinnamon, LXQt, LXDE, and Openbox as installable desktop choices instead of embedding every desktop in the ISO.
- Embeds checksum-verified official Waydroid GAPPS system and MAINLINE vendor images.
- Initializes Waydroid during OS installation and verifies its container before the first graphical login.
- Includes Google Play services and the Google Play Store. Google may require device registration using [Waydroid's Play certification procedure](https://docs.waydro.id/faq/google-play-certification).
- Installs APK, APKS, and APKM packages, including atomic split-APK installation.
- Offers explicit first-login ARM32 and ARM64 translation setup. Proprietary translation payloads are downloaded directly to the installed machine after consent; they are not included in the ISO.
- Integrates Android applications into desktop menus through the droidianOS integration service.
- Separates package, repository, ISO, release, archive, and SourceForge publishing stages in one GitHub Actions workflow.

## Branding

The droidianOS identity represents a desktop host and a controlled bridge for Android application integration. One canonical flat geometric mark is used across all logo variants: a cyan host-shaped `D` crossed by a restrained amber bridge.

The branding package applies this identity to:

- distribution metadata, console identification, and MOTD
- live ISO and installed bootloader menus
- GRUB and desktop-base backgrounds
- Plymouth startup artwork and animation
- LightDM login presentation
- desktop and lock-screen wallpaper defaults
- application, vendor, and scalable icon assets

Canonical assets are maintained in `packages/droidianos-branding`. Raster variants are generated from the canonical SVG sources during package builds where practical.

## Build Packages

On Debian Stable or a compatible build container:

```sh
sudo apt-get update
sudo apt-get install -y build-essential cargo devscripts debhelper dpkg-dev fonts-dejavu-core libdbus-1-dev librsvg2-bin rustc
sh ./build/scripts/build-packages.sh
```

Package artifacts are written to `build/artifacts/packages`.

## Build ISO

After installing the package-build dependencies above, install the additional ISO tools:

```sh
sudo apt-get install -y apt-utils curl gnupg isolinux jq live-build squashfs-tools syslinux-common unzip xorriso
sh ./build/scripts/build-packages.sh
sh ./build/scripts/build-local-repo.sh
sh ./build/scripts/build-iso.sh
```

The ISO build resolves the current Debian Stable codename, configures the corresponding official Waydroid APT archive, and downloads the latest verified x86-64 GAPPS and MAINLINE images. ISO artifacts are written to `build/artifacts/iso`.

## Release

```sh
DROIDIANOS_GPG_KEY=<key-id> sh ./build/scripts/release-artifacts.sh
```

Pushing to GitHub runs `.github/workflows/build.yml`. Pull requests build packages only. Pushes and manual runs continue through repository creation, ISO generation, release packaging, archival, and SourceForge publication.

Required repository secrets:

- `SOURCEFORGE_UPLOAD_TARGET`: SSH/rsync destination for the SourceForge ISO directory.
- `SOURCEFORGE_SSH_PRIVATE_KEY`: private SSH key authorized for SourceForge uploads.

Optional repository secrets:

- `DROIDIANOS_GPG_PRIVATE_KEY`: ASCII-armored release-signing private key.
- `DROIDIANOS_GPG_KEY`: GPG key ID used for repository and checksum signatures.
- `DROIDIANOS_GPG_PASSPHRASE`: signing-key passphrase, when required.

## Credits

droidianOS builds on work from:

- [Debian](https://www.debian.org/) and Debian Live for the base operating system, packaging ecosystem, and live-build tooling.
- [Waydroid](https://waydro.id/) for the GPL-3.0 Android container runtime and official image distribution.
- [LineageOS](https://lineageos.org/) and the [Android Open Source Project](https://source.android.com/) for the Android system foundation used by Waydroid images.
- Google for Google Play services and the Google Play Store included by the upstream Waydroid GAPPS image. These components are proprietary and are not authored or licensed by droidianOS.
- [casualsnek/waydroid_script](https://github.com/casualsnek/waydroid_script) for the GPL-3.0 optional ARM installer logic, and the referenced upstream translation payload projects for the compatibility binaries.
- Labwc, LightDM, Plymouth, desktop-base, GNOME, KDE, Xfce, Cinnamon, LXQt, LXDE, Openbox, and their contributors for the supported Linux desktop stack.
- The [DejaVu Fonts](https://dejavu-fonts.github.io/) project for typography used when generating branding assets.

All trademarks and third-party names remain the property of their respective owners. droidianOS is not affiliated with or endorsed by Debian, Google, Waydroid, LineageOS, or the listed desktop projects.

## License

Original droidianOS repository code and artwork are licensed under the MIT License unless a file states otherwise. The generated distribution aggregates independently licensed and proprietary components; the MIT License does not relicense them. See `LICENSE`.
