# droidianOS Ubuntu, KDE Plasma, and Waydroid Compatibility Plan

## 1. Objective

Migrate the distribution to:

- The latest released Ubuntu LTS at ISO build time as the base operating system.
- KDE Plasma on Wayland as the sole desktop environment.
- SDDM as the sole display manager.
- Waydroid as a hidden Android runtime.
- The existing Waydroid ARM-translation setup as compatibility support for ARM Android applications.

Android applications must continue to install, launch, notify, and appear as normal KDE applications. Waydroid must not expose an Android launcher or separate user-facing desktop flow.

## 2. Verified Findings

### 2.1 Ubuntu target policy

Resolve the latest released Ubuntu LTS at the start of each ISO build. Freeze its version, codename, repository URLs, package versions, and build timestamp in the release manifest. Do not silently change the base during a release build.

### 2.2 Current base and image build

The ISO build is Debian-specific:

- `build/live-build/auto/config` uses Debian Stable, Debian archive areas, and the Debian installer.
- `build/live-build/config/package-lists/droidianos.list.chroot` installs Labwc, LightDM, the LightDM GTK greeter, Xorg, and the Debian installer launcher.
- `build/scripts/build-iso.sh` downloads Waydroid images before building the ISO.
- CI uses a `debian-stable` runner.

### 2.3 Current desktop selector

`packages/droidianos-session-defaults` contains a selector offering KDE Plasma, GNOME, Xfce, Cinnamon, LXQt, LXDE, and Openbox. Its executable, Python implementation, desktop entry, preview assets, and installer-hub wording must be removed.

### 2.4 Current Android implementation

Waydroid is directly used by:

- `droidianos-waydroid-image` for images, first boot, system services, shared folders, and ARM setup.
- `droidianos-apk-installer` for APK installation and package verification.
- `droidianos-integrationd` for package discovery, launching, and notifications.
- `droidianos-settings` for permissions, diagnostics, and recovery.
- `droidianos-software-center` for Android application removal.
- `droidianos-session-defaults` for runtime startup and hiding direct launcher entries.

The existing ARM setup installs Waydroid-specific translation payloads. It is retained as a compatibility feature, not presented as a guarantee that every Android application will run.

## 3. Target Architecture

| Area | Target |
| --- | --- |
| Base distribution | Latest released Ubuntu LTS at ISO build time |
| Desktop | KDE Plasma Wayland only |
| Display manager | SDDM only |
| Desktop selector | Removed |
| Android runtime | Waydroid, hidden from normal user flow |
| ARM compatibility | Existing Waydroid ARM setup, validated per ABI and app |
| Native Android ABIs | x86 and x86_64 |
| Translated Android ABIs | armeabi-v7a and arm64-v8a where validated |
| App presentation | Existing desktop-entry, icon, launch, notification, and settings integration |
| Packaging | Ubuntu `.deb` packages and signed project APT repository |

## 4. Delivery Gates

### Gate A: Ubuntu base proof-of-concept

Tasks:

1. Resolve the latest released Ubuntu LTS and configure its repositories and codename.
2. Replace Debian-installer assumptions with an Ubuntu-compatible installer flow.
3. Update repository metadata, archive configuration, release manifests, and CI base image.
4. Build a clean Ubuntu-based development ISO.

Exit criteria:

- ISO boots in a clean VM.
- Installed system identifies as derived from the resolved Ubuntu LTS.
- Installation completes.
- APT uses repositories for the resolved Ubuntu LTS and the signed project repository.
- Release metadata records the resolved LTS version and codename.

### Gate B: KDE-only desktop proof-of-concept

Tasks:

1. Install Plasma Wayland and SDDM in the ISO package list.
2. Remove Labwc, LightDM, LightDM GTK greeter, Xorg fallback, Openbox, and desktop-selector dependencies.
3. Delete selector code, preview assets, selector desktop entry, selector launcher, and selector package-install rules.
4. Rewrite installer and first-login text to state KDE Plasma is preinstalled.
5. Retain only Plasma-compatible branding and session defaults.

Exit criteria:

- SDDM starts Plasma Wayland after installation.
- No selector executable, service, launcher, asset, or alternate-desktop task is installed.
- The live ISO and installed system provide only KDE Plasma.

### Gate C: Waydroid preservation on Ubuntu

Tasks:

1. Verify the Waydroid package source and supported package version for the resolved Ubuntu LTS.
2. Port `droidianos-waydroid-image` package dependencies, first-boot initialization, image handling, services, and shared-folder policy to Ubuntu.
3. Retain the existing image-fetch and ARM-setup scripts only after pinning their sources, hashes, and compatible Android version.
4. Preserve headless/lazy session start and direct-launch behavior.
5. Preserve hiding of Waydroid launcher entries from KDE menus.

Exit criteria:

- Waydroid initializes on a clean KDE installation based on the resolved Ubuntu LTS.
- No Android home screen or Waydroid launcher appears in normal flow.
- Existing project packages operate without replacing their Waydroid command contracts.
- Runtime reset does not affect host files outside documented Waydroid directories.

### Gate D: ARM compatibility validation

Test fixtures:

| ABI | Required fixtures |
| --- | --- |
| x86 | Single APK, split APK, JNI app |
| x86_64 | Single APK, split APK, JNI app |
| armeabi-v7a | Single APK, split APK, JNI app |
| arm64-v8a | Single APK, split APK, JNI app |

For every fixture, verify:

1. APK inspection and permission display.
2. Installation and package discovery.
3. Desktop-entry and icon generation.
4. First and subsequent launch.
5. Window rendering, input, sound, network, and storage access.
6. Lifecycle transitions and Waydroid restart recovery.
7. Notifications where emitted.
8. Permission grant and revoke where requested.
9. Uninstall and desktop-entry removal.

Exit criteria:

- Native x86/x86_64 fixtures pass.
- ARMv7 and ARM64 results are recorded separately.
- Public compatibility claims include only passing combinations.
- Failed applications are not presented as supported.

### Gate E: Integration and release readiness

Tasks:

1. Retain and test the existing APK installer, integration daemon, settings, software center, diagnostics, recovery, notification bridge, and shared-folder policy.
2. Replace Debian-specific dependencies and assumptions in each package with equivalents compatible with the resolved Ubuntu LTS.
3. Update documentation, release notes, manifests, security model, and license notices.
4. Remove references to alternate desktop environments and the desktop selector.
5. Keep Waydroid references only where technically required; normal user-facing text must describe installed Android applications, not a separate runtime.
6. Build packages and the ISO in a clean Ubuntu environment.

Exit criteria:

- Clean install, update, Android application install, launch, uninstall, diagnostics, and recovery pass.
- No Debian or alternate-desktop implementation path remains in release artifacts.
- Waydroid remains hidden while Android applications remain integrated with KDE.

## 5. Required Changes by Area

### 5.1 ISO, CI, and repository

Affected areas:

- `build/live-build/auto/config`
- `build/live-build/config/package-lists/droidianos.list.chroot`
- `build/live-build/config/hooks/`
- `build/scripts/build-iso.sh`
- `build/scripts/build-local-repo.sh`
- `ci/pipelines/package-build.yml`
- `build/debian-repo/`

Required result: Bootstrap, repositories, installer, package availability, and CI are functional for the resolved Ubuntu LTS before Android changes are made.

### 5.2 Desktop defaults

Affected areas:

- `packages/droidianos-session-defaults/`
- Branding defaults that mention multiple desktop environments.
- Live-session startup configuration.
- Installer-hub text and installed-session startup.

Required result: delete the desktop selector. Plasma Wayland and SDDM are the single graphical path.

### 5.3 Android integration

Affected areas:

- `packages/droidianos-waydroid-image/`
- `packages/droidianos-apk-installer/`
- `packages/droidianos-integrationd/`
- `packages/droidianos-settings/`
- `packages/droidianos-software-center/`
- `packages/droidianos-session-defaults/`

Required result: retain the existing Waydroid integration contract and port only the host-distribution dependencies and service assumptions needed for Ubuntu.

### 5.4 Documentation and release metadata

Affected areas:

- `README.md`
- `LICENSE`
- `docs/`
- `docs/releases/`
- Package README files.
- Release manifest templates.

Required result: remove Debian and desktop-selector claims. Keep Waydroid technical references where required. Do not claim universal ARM compatibility.

## 6. Risks and Controls

| Risk | Control |
| --- | --- |
| Waydroid package unavailable for the resolved Ubuntu LTS | Validate package source and installability at Gate C. |
| ARM translator payload cannot be redistributed or updated safely | Pin sources and hashes; verify licensing before release. |
| ARMv7 or ARM64 app fails | Record ABI-specific results; do not claim support. |
| GPU translation failure | Test graphics fixtures on each supported GPU class. |
| Ubuntu package difference breaks project packages | Build each package in a clean environment matching the resolved Ubuntu LTS. |
| KDE-only requirement regresses through dependencies | Audit installed packages and desktop sessions in the final ISO. |
| Waydroid becomes user-visible | Test menus, login, install, launch, and recovery flows. |

## 7. Completion Definition

The migration is complete only when:

1. The release ISO is based on the latest Ubuntu LTS released when its build began, recorded in its manifest.
2. KDE Plasma Wayland is the only installed and selectable desktop session.
3. SDDM is the only display manager.
4. The desktop selector and all alternate-desktop assets are absent.
5. Waydroid runs as the hidden Android backend.
6. Android apps integrate with KDE through existing project-owned installer, launcher, notification, settings, and software-center paths.
7. x86/x86_64, ARMv7, and ARM64 compatibility is documented only from validated results.
8. Documentation describes verified limitations and does not promise universal Android-app compatibility.

## 8. Immediate Next Action

Implement build-time latest-LTS resolution and manifest recording, then validate Waydroid and the existing ARM-translation payload against the resolved LTS before modifying production packages.
