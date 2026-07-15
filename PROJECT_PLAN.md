# droidianOS Project Plan

## 1. Overall System Architecture

### Goal

Build a Debian Stable based desktop operating system where Android applications behave like normal Linux desktop applications. Android execution is provided by Waydroid. The distribution owns integration, packaging, installer flow, settings, updates, and polish.

### Layers

```text
User
  |
Desktop shell, launcher, notifications, settings, software center
  |
Integration services over D-Bus
  |
Waydroid session, Android image, microG, app bridge
  |
Debian Stable, systemd, Wayland, PipeWire, Flatpak, APT
  |
Linux kernel, Mesa, hardware drivers
```

### Main Components

| Component | Responsibility |
|---|---|
| Base OS | Debian Stable, systemd boot, Wayland desktop, PipeWire, APT, Flatpak |
| Android runtime | Waydroid container, desktop-optimized Android image, microG |
| Integration daemon | App discovery, desktop entries, icons, MIME, permissions, notification sync |
| APK installer backend | Parse APK metadata, show permissions, install into Waydroid, emit integration events |
| Software Center | Unified install/update/remove UI for APT, Flatpak, APK, and local packages |
| Settings app | Unified Linux and Android settings surface |
| Update service | Unified update status and operations across APT, Flatpak, Android image, APK apps |
| Build system | Image generation, package repository, ISO generation, CI validation |

### Design Constraints

- Do not fork or replace Waydroid.
- Do not build a new Android runtime.
- Do not expose Android as a separate user-facing subsystem.
- Treat every Android app as an installed application.
- Use D-Bus contracts between GUI apps and privileged services.
- Keep privileged code small and auditable.

## 2. Repository Structure

```text
.
|-- README.md
|-- docs/
|   |-- architecture.md
|   |-- dbus-api.md
|   |-- security-model.md
|   |-- packaging.md
|   |-- testing.md
|   `-- release-process.md
|-- packages/
|   |-- android-integration-daemon/
|   |-- apk-installer-backend/
|   |-- software-center/
|   |-- settings-app/
|   |-- update-service/
|   |-- waydroid-image-customizations/
|   |-- desktop-session-defaults/
|   `-- distro-branding/
|-- shared/
|   |-- dbus-interfaces/
|   |-- rust-libs/
|   |-- flutter-widgets/
|   `-- policy/
|-- build/
|   |-- live-build/
|   |-- debian-repo/
|   |-- iso/
|   `-- scripts/
|-- tests/
|   |-- integration/
|   |-- e2e/
|   |-- package/
|   `-- fixtures/
|-- ci/
|   |-- pipelines/
|   |-- vm-tests/
|   `-- release/
`-- tools/
    |-- apk-metadata/
    |-- icon-extract/
    `-- image-signing/
```

### Ownership

| Path | Owner |
|---|---|
| `packages/android-integration-daemon` | System integration |
| `packages/apk-installer-backend` | APK installation |
| `packages/software-center` | App management UI |
| `packages/settings-app` | Settings UI |
| `packages/update-service` | Updates |
| `build` | OS image and ISO |
| `shared/dbus-interfaces` | Stable IPC contracts |

## 3. Development Roadmap

### Phase 0: Feasibility

- Boot Debian Stable with target Wayland session.
- Install and launch Waydroid manually.
- Verify GPU acceleration, clipboard, sound, networking, and app windows.
- Verify microG image build path.
- Define supported hardware baseline.

### Phase 1: MVP Desktop Integration

- Package Waydroid defaults.
- Create integration daemon.
- Detect installed Android apps.
- Generate `.desktop` files.
- Extract icons.
- Launch Android apps directly from Linux launcher.
- Hide Android launcher from normal flow.

### Phase 2: APK Install Flow

- Register APK MIME type.
- Create APK metadata parser.
- Add installer backend.
- Show app name, version, icon, permissions, publisher if available, and storage estimate.
- Install APK into Waydroid.
- Refresh desktop entries and search index.

### Phase 3: Unified App Management

- Build Software Center with APT, Flatpak, APK, and local package backends.
- Add uninstall and update flows.
- Add app detail pages.
- Add permission management.
- Add app lifecycle state.

### Phase 4: Settings and Security

- Build permissions UI.
- Add shared-folder controls.
- Add autostart and background controls.
- Add notification controls.
- Add Android image update controls.
- Add user-facing diagnostics.

### Phase 5: Production Image

- Build signed package repository.
- Build ISO.
- Add automated VM install tests.
- Add upgrade tests.
- Add recovery tooling.
- Add release process.

## 4. Milestone Plan

| Milestone | Output | Exit Criteria |
|---|---|---|
| M0 | Technical spike | Waydroid runs on Debian Stable Wayland with GPU acceleration |
| M1 | Base image | Bootable desktop ISO with Waydroid installed but hidden |
| M2 | App discovery | Installed Android apps appear in launcher with icons |
| M3 | App launch | Android apps launch directly as individual desktop windows |
| M4 | APK install | Double-click APK opens installer and installs app |
| M5 | Software Center MVP | APT, Flatpak, and APK apps install/remove from one UI |
| M6 | Permissions MVP | Camera, mic, storage, location, notifications, autostart visible per app |
| M7 | Updates MVP | Unified update list for APT, Flatpak, Android image, APK apps |
| M8 | Security hardening | Shared folders, permission prompts, confinement rules documented and tested |
| M9 | Beta ISO | Upgradeable signed ISO with CI-tested install path |
| M10 | 1.0 | Stable release with recovery and telemetry-free diagnostics |

## 5. Package Architecture

### Debian Packages

| Package | Type | Contents |
|---|---|---|
| `droidianos-session-defaults` | Architecture independent | Desktop defaults, MIME registrations, launcher policies |
| `droidianos-integrationd` | Architecture dependent | Rust daemon and systemd user service |
| `droidianos-apk-installer` | Architecture dependent | APK parser, installer service, polkit rules |
| `droidianos-software-center` | Architecture dependent | Flutter desktop app |
| `droidianos-settings` | Architecture dependent | Flutter desktop settings app |
| `droidianos-update-service` | Architecture dependent | Update orchestration daemon |
| `droidianos-waydroid-image` | Architecture independent | Image metadata, microG integration, OTA channel config |
| `droidianos-branding` | Architecture independent | Wallpapers, icons, Plymouth, appstream metadata |

### Package Rules

- Every privileged operation must go through D-Bus plus polkit.
- GUI apps must not run as root.
- Generated `.desktop` files must be owned by the user.
- Runtime state must live under XDG paths.
- Package maintainer scripts must be idempotent.

### Project APT Repository

Project-owned packages are published to a dedicated signed APT repository, not directly to Debian's official archive during early releases.

Repository tools:

- Preferred: `aptly`.
- Acceptable for early development: `reprepro`.
- Development-only fallback: `dpkg-scanpackages`.

Repository layout:

```text
https://repo.example.org/debian/
|-- dists/stable/InRelease
|-- dists/stable/Release
|-- dists/stable/Release.gpg
|-- dists/stable/main/binary-amd64/Packages
|-- dists/stable/main/binary-amd64/Packages.gz
`-- pool/main/n/
```

User systems install:

```text
/etc/apt/keyrings/droidianos.gpg
/etc/apt/sources.list.d/droidianos.list
```

Example source:

```text
deb [signed-by=/etc/apt/keyrings/droidianos.gpg] https://repo.example.org/debian stable main
```

Upload flow:

```text
1. CI builds .deb packages from clean source.
2. CI runs package, service, and VM smoke tests.
3. CI imports the release signing key from protected secrets.
4. CI publishes packages into the staged APT repository.
5. CI signs Release metadata and produces InRelease.
6. CI uploads repository contents to repo hosting.
7. CI runs apt update/upgrade smoke test against the hosted repository.
8. CI promotes the repository from staging to stable.
```

Official Debian archive submission is a later upstreaming effort and requires Debian policy compliance, maintainership, review, and sponsorship.

### Runtime Paths

```text
~/.local/share/applications/android-*.desktop
~/.local/share/icons/hicolor/*/apps/android-*.png
~/.local/share/droidianos/apps.json
~/.config/droidianos/permissions.json
~/.cache/droidianos/apk-metadata/
/usr/share/dbus-1/interfaces/
/usr/share/polkit-1/actions/
/var/lib/droidianos/
```

## 6. Installer Architecture

### APK MIME Registration

Register:

```text
application/vnd.android.package-archive
```

Associate with:

```text
droidianos-apk-installer.desktop
```

### Installer Flow

```text
File manager double-click
  -> software installer opens APK
  -> APK parser extracts metadata
  -> installer UI renders review screen
  -> user clicks Install
  -> backend validates package
  -> Waydroid session starts headless if needed
  -> APK installed with waydroid app install
  -> integration daemon refreshes app registry
  -> desktop entry and icon generated
  -> launcher/search updated
```

### Installer UI Data

| Field | Source |
|---|---|
| App name | APK manifest label and resources |
| Version | APK manifest |
| Icon | APK resources |
| Permissions | APK manifest |
| Publisher | APK signing certificate when useful |
| Storage estimate | APK size plus extracted size estimate |

### Failure Handling

| Failure | User Message |
|---|---|
| Invalid APK | This file is not a valid application package. |
| Unsupported ABI | This application is not compatible with this computer. |
| Waydroid unavailable | Application service is starting. Try again in a moment. |
| Install failed | Installation failed. Details are available in diagnostics. |
| Dangerous permission | Show explicit permission warning before install |

## 7. Desktop Integration Architecture

### Integration Daemon

Name:

```text
droidianos-integrationd
```

Language:

```text
Rust
```

Mode:

```text
systemd user service
```

Responsibilities:

- Watch Waydroid package list.
- Maintain app registry.
- Generate `.desktop` entries.
- Extract and cache icons.
- Sync app names.
- Remove stale launcher entries.
- Manage MIME associations.
- Expose app lifecycle over D-Bus.
- Bridge notifications.
- Apply permission policy.

### Desktop Entry Template

```ini
[Desktop Entry]
Type=Application
Name={{app_name}}
Exec=droidianos-launch {{package_name}} {{activity_name}}
Icon={{icon_name}}
Categories={{categories}}
StartupNotify=true
X-DroidianOS-Package={{package_name}}
X-DroidianOS-Activity={{activity_name}}
```

### Launch Flow

```text
User clicks app
  -> droidianos-launch package/activity
  -> integration daemon ensures Waydroid session
  -> daemon launches target activity
  -> Wayland compositor shows app window
  -> lifecycle state changes to Running
```

### App Registry

```json
{
  "package": "com.example.app",
  "name": "Example",
  "version": "1.2.3",
  "activities": [
    {
      "name": "com.example.MainActivity",
      "launchable": true
    }
  ],
  "icon": "android-com.example.app",
  "source": "apk",
  "installed_at": "2026-01-01T00:00:00Z",
  "permissions": []
}
```

## 8. D-Bus Service Definitions

### Service Names

```text
org.droidianos.Integration
org.droidianos.Installer
org.droidianos.Permissions
org.droidianos.Updates
```

### `org.droidianos.Integration`

```xml
<node>
  <interface name="org.droidianos.Integration">
    <method name="ListApps">
      <arg name="apps_json" type="s" direction="out"/>
    </method>
    <method name="RefreshApps"/>
    <method name="LaunchApp">
      <arg name="package" type="s" direction="in"/>
      <arg name="activity" type="s" direction="in"/>
    </method>
    <method name="UninstallApp">
      <arg name="package" type="s" direction="in"/>
    </method>
    <signal name="AppsChanged"/>
    <signal name="AppStateChanged">
      <arg name="package" type="s"/>
      <arg name="state" type="s"/>
    </signal>
  </interface>
</node>
```

### `org.droidianos.Installer`

```xml
<node>
  <interface name="org.droidianos.Installer">
    <method name="InspectApk">
      <arg name="path" type="s" direction="in"/>
      <arg name="metadata_json" type="s" direction="out"/>
    </method>
    <method name="InstallApk">
      <arg name="path" type="s" direction="in"/>
      <arg name="transaction_id" type="s" direction="out"/>
    </method>
    <method name="GetTransaction">
      <arg name="transaction_id" type="s" direction="in"/>
      <arg name="status_json" type="s" direction="out"/>
    </method>
    <signal name="InstallProgress">
      <arg name="transaction_id" type="s"/>
      <arg name="percent" type="u"/>
      <arg name="message" type="s"/>
    </signal>
  </interface>
</node>
```

### `org.droidianos.Permissions`

```xml
<node>
  <interface name="org.droidianos.Permissions">
    <method name="ListPermissions">
      <arg name="package" type="s" direction="in"/>
      <arg name="permissions_json" type="s" direction="out"/>
    </method>
    <method name="SetPermission">
      <arg name="package" type="s" direction="in"/>
      <arg name="permission" type="s" direction="in"/>
      <arg name="state" type="s" direction="in"/>
    </method>
    <signal name="PermissionsChanged">
      <arg name="package" type="s"/>
    </signal>
  </interface>
</node>
```

### `org.droidianos.Updates`

```xml
<node>
  <interface name="org.droidianos.Updates">
    <method name="CheckUpdates">
      <arg name="transaction_id" type="s" direction="out"/>
    </method>
    <method name="ListUpdates">
      <arg name="updates_json" type="s" direction="out"/>
    </method>
    <method name="ApplyUpdates">
      <arg name="update_ids" type="as" direction="in"/>
      <arg name="transaction_id" type="s" direction="out"/>
    </method>
    <signal name="UpdatesChanged"/>
    <signal name="UpdateProgress">
      <arg name="transaction_id" type="s"/>
      <arg name="percent" type="u"/>
      <arg name="message" type="s"/>
    </signal>
  </interface>
</node>
```

## 9. Software Center Specification

### Scope

One GUI for discovering, installing, updating, and removing:

- APT packages.
- Flatpak applications.
- Local `.deb` files.
- Android APK files.

### UI Principles

- Do not expose backend names on primary screens.
- Show source only in details or advanced metadata.
- Use one install button.
- Use one update list.
- Show permissions before installing APK apps.

### Screens

| Screen | Purpose |
|---|---|
| Home | Featured apps, categories, recent updates |
| Search | Unified search across APT metadata, Flatpak metadata, local index, APK file when opened |
| App details | Name, icon, screenshots where available, description, version, source, permissions |
| Installed | All installed apps with remove/manage actions |
| Updates | Unified updates |
| APK review | Local APK install confirmation |

### Backend Interface

```text
trait AppBackend {
  search(query) -> Vec<AppSummary>
  inspect(id) -> AppDetails
  install(id) -> Transaction
  remove(id) -> Transaction
  list_installed() -> Vec<AppSummary>
  list_updates() -> Vec<Update>
  update(ids) -> Transaction
}
```

### Backend Selection

| Input | Backend |
|---|---|
| Search result from APT metadata | APT |
| Search result from Flatpak metadata | Flatpak |
| `.deb` file | Debian package backend |
| `.apk` file | APK installer backend |
| Installed Android app | Android app backend |

## 10. Settings Application Specification

### Screens

| Screen | Contents |
|---|---|
| Display | Resolution, scale, night light, monitors |
| Audio | Devices, volume, microphone |
| Network | Wi-Fi, Ethernet, VPN |
| Applications | Installed apps, defaults, startup |
| Permissions | Camera, microphone, storage, location, notifications, background, autostart, network |
| Storage | Disk usage, Android app data, shared folders |
| Accounts | Online accounts and microG-related account surface where supported |
| Updates | OS, apps, Android image |
| Diagnostics | Logs, service health, export report |

### Android App Settings

Each app detail page:

- Open.
- Force stop.
- Uninstall.
- Storage usage.
- Clear app data.
- Permissions.
- Notifications.
- Background execution.
- Autostart.
- Shared folders.

## 11. APK Installation Pipeline

```text
1. File selected.
2. MIME type confirms APK.
3. APK parser validates ZIP and manifest.
4. Metadata extracted.
5. Signature inspected.
6. ABI compatibility checked.
7. Permissions classified.
8. User confirms.
9. Installer starts Waydroid session if required.
10. APK copied to staging path.
11. APK installed through Waydroid.
12. Package manager state verified.
13. Integration daemon refreshes registry.
14. Icon extracted and cached.
15. Desktop entry generated.
16. Search index refreshed.
17. Install transaction marked complete.
```

### Permission Classification

| Class | Examples | UI Behavior |
|---|---|---|
| Normal | Internet, vibration | Listed |
| Sensitive | Camera, mic, location, contacts, storage | Highlighted |
| Background | autostart, background service | Explicit toggle |
| Unsupported | privileged Android-only permissions | Warn or block |

## 12. Android Application Lifecycle

### States

```text
NotInstalled
Installed
Starting
Running
Background
Stopped
Updating
Uninstalling
Failed
```

### Lifecycle Events

| Event | Source | Result |
|---|---|---|
| APK installed | Installer | Registry refresh |
| App launched | Desktop entry | Waydroid activity start |
| Window opened | Wayland/compositor observation | State Running |
| App backgrounded | Android lifecycle | State Background |
| App stopped | Android lifecycle | State Stopped |
| App uninstalled | Installer or Android package manager | Desktop entry removed |
| App updated | APK or store source | Registry and icon refresh |

### Startup

- Waydroid should not show a launcher.
- Waydroid session may start lazily on first app launch.
- Frequently used apps may be prewarmed if the user enables fast startup.
- Autostart apps are controlled in Linux startup settings.

## 13. Security Model

### Threat Model

| Threat | Mitigation |
|---|---|
| APK reads all user files | Default deny, explicit shared folders only |
| APK abuses camera or mic | Permission prompt and settings control |
| APK runs in background silently | Background execution controls |
| APK spoofs native app | Source and package identity visible in details |
| APK persists unwanted autostart | Startup manager integration |
| Malicious APK exploits bridge | Minimize bridge API and validate inputs |
| Privilege escalation through installer | Polkit, unprivileged GUI, audited privileged helper |

### Filesystem Policy

- No unrestricted home directory mount by default.
- Provide shared folders explicitly.
- Default shared folders:
  - Downloads: optional.
  - Pictures: optional.
  - Documents: optional.
- Use per-app allowlist where technically possible.

### Process Policy

- Integration daemon runs as user.
- System updates require polkit.
- APK install into user Waydroid session should not require root after setup.
- Privileged helpers must have narrow commands and strict argument validation.

### Network Policy

- Network access is allowed by default for compatibility.
- Per-app network blocking is a later milestone unless a mature backend is available.

## 14. UX Mockups

### APK Install

```text
+------------------------------------------------+
| Install WhatsApp                               |
|                                                |
| [icon] WhatsApp                                |
|        Version 2.x                             |
|        Publisher: WhatsApp LLC                 |
|                                                |
| Permissions                                    |
|  Camera        Required                        |
|  Microphone    Required                        |
|  Contacts      Required                        |
|  Notifications Optional                        |
|                                                |
| Storage: about 220 MB                          |
|                                                |
|                         [Cancel] [Install]     |
+------------------------------------------------+
```

### Software Center App Page

```text
+------------------------------------------------+
| Search: whatsapp                               |
|------------------------------------------------|
| [icon] WhatsApp                                |
| Messaging                                      |
|                                                |
| [Install]                                      |
|                                                |
| Details                                        |
| Version: 2.x                                   |
| Source: Application package                    |
| Permissions: Camera, Microphone, Contacts      |
+------------------------------------------------+
```

### App Permissions

```text
+------------------------------------------------+
| WhatsApp                                       |
|------------------------------------------------|
| Camera                  [on]                   |
| Microphone              [on]                   |
| Location                [off]                  |
| Notifications           [on]                   |
| Background execution    [on]                   |
| Start automatically     [off]                  |
| Shared folders          Downloads              |
+------------------------------------------------+
```

### Unified Updates

```text
+------------------------------------------------+
| Updates                                        |
|------------------------------------------------|
| System packages             12 updates         |
| Desktop applications         3 updates         |
| Application support image    1 update          |
| WhatsApp                     1 update          |
|                                                |
|                              [Update All]      |
+------------------------------------------------+
```

## 15. API Documentation

### App Summary

```json
{
  "id": "android:com.example.app",
  "name": "Example",
  "source": "apk",
  "icon": "android-com.example.app",
  "installed": true,
  "version": "1.0.0"
}
```

### App Details

```json
{
  "id": "android:com.example.app",
  "package": "com.example.app",
  "name": "Example",
  "version": "1.0.0",
  "source": "apk",
  "permissions": [
    {
      "id": "android.permission.CAMERA",
      "label": "Camera",
      "class": "sensitive",
      "state": "allowed"
    }
  ],
  "storage_bytes": 230000000,
  "launchable": true
}
```

### Transaction

```json
{
  "id": "tx-123",
  "kind": "install",
  "state": "running",
  "percent": 42,
  "message": "Installing application"
}
```

### Update

```json
{
  "id": "android:com.example.app:1.0.1",
  "name": "Example",
  "source": "apk",
  "current_version": "1.0.0",
  "new_version": "1.0.1",
  "size_bytes": 80000000
}
```

## 16. CI/CD Pipeline

### Pipeline Stages

```text
lint
unit-test
package-build
integration-test
vm-install-test
iso-build
iso-boot-test
upgrade-test
sign
publish
```

### Required Checks

| Check | Scope |
|---|---|
| Rust tests | Daemons and parsers |
| Flutter tests | Software Center and Settings |
| Debian package lint | All `.deb` packages |
| D-Bus ABI check | Interface compatibility |
| VM smoke test | Boot, login, launch native app |
| Waydroid smoke test | Start session, install APK fixture, launch app |
| ISO test | Boot installer and live session |
| Upgrade test | Previous release to current release |

### Artifacts

- `.deb` packages.
- Signed APT repository.
- Waydroid image metadata.
- Bootable ISO.
- VM test logs.
- SBOM.
- Checksums.

### Repository Publishing

CI publishes all project packages to the signed project APT repository after tests pass. The repository is the update channel for OS components created by this project. Debian Stable and Debian security repositories remain the source for base OS packages.

Required CI secrets:

- APT repository signing key.
- Repository upload credential.
- Staging repository URL.
- Stable repository URL.

Publishing checks:

- `apt update` succeeds against staging.
- `apt-cache policy droidianos-integrationd` shows the staged version.
- Upgrade from previous release succeeds in a VM.
- Repository metadata signatures verify with `/etc/apt/keyrings/droidianos.gpg`.

## 17. Automated Testing Strategy

### Unit Tests

- APK metadata parser.
- D-Bus payload serialization.
- Desktop entry generation.
- Permission classification.
- Update transaction state machine.

### Integration Tests

- Install APK fixture.
- Generate launcher entry.
- Launch app.
- Remove app.
- Refresh registry.
- Permission toggle persistence.

### End-to-End VM Tests

```text
1. Boot ISO.
2. Create user.
3. Open file manager.
4. Double-click APK fixture.
5. Install app.
6. Confirm launcher entry exists.
7. Launch app.
8. Confirm window appears.
9. Toggle notification permission.
10. Uninstall app.
11. Confirm desktop entry removed.
```

### Performance Tests

| Metric | Target |
|---|---|
| Cold app launch | Track and reduce per milestone |
| Warm app launch | Track and reduce per milestone |
| Idle CPU | Near zero when no Android app is running |
| Memory overhead | Measured with and without Waydroid session |
| ISO boot time | Track per release |

## 18. Build System

### Tooling

- Debian packaging with `dpkg-buildpackage`.
- Local signed APT repository.
- `live-build` for ISO generation.
- Python build orchestration scripts.
- Rust build through Cargo.
- Flutter Linux builds for GUI apps.

### Build Flow

```text
1. Build shared libraries.
2. Build Rust daemons.
3. Build Flutter apps.
4. Build Debian packages.
5. Publish packages into local APT repo.
6. Build customized Waydroid image metadata.
7. Build live ISO with package repo enabled.
8. Run VM boot tests.
9. Sign artifacts.
```

### Versioning

Use one distribution release version plus package versions:

```text
OS release: 1.0
Package version: 1.0.0-1
Image channel: stable
```

## 19. ISO Generation Process

### Inputs

- Debian Stable package repositories.
- Project APT repository.
- Desktop environment packages.
- Waydroid packages.
- Integration packages.
- Branding packages.
- Installer configuration.

### Process

```text
1. Create clean build environment.
2. Import signing keys.
3. Configure Debian Stable repositories.
4. Add project repository.
5. Configure package list.
6. Configure desktop defaults.
7. Configure Waydroid defaults.
8. Configure live user and installer.
9. Build ISO with live-build.
10. Boot ISO in VM.
11. Run smoke tests.
12. Generate checksum.
13. Sign checksum.
14. Publish ISO and metadata.
```

### ISO Acceptance Criteria

- Boots in UEFI VM.
- Live session starts.
- Installer starts.
- Installed system boots.
- User can log in.
- Wayland session is default.
- APK MIME association exists.
- Waydroid service can initialize.
- Sample APK fixture can install and launch.

## 20. Incremental AI-Assisted Implementation Plan

### Work Unit Rules

- One package or service per branch.
- One D-Bus interface change per review.
- No GUI work until backend contract exists.
- Every generated file must have an owner package.
- Every milestone must boot in a VM.

### Increment 1: Base Repository

Deliver:

- Repository skeleton.
- Build scripts.
- Package templates.
- CI skeleton.

Acceptance:

- Empty packages build.
- CI runs package build job.

### Increment 2: Waydroid Baseline Package

Deliver:

- Package installing Waydroid defaults.
- systemd service configuration.
- First boot initialization script.

Acceptance:

- Clean VM can initialize Waydroid.
- No Android launcher is exposed by default.

### Increment 3: App Registry

Deliver:

- `droidianos-integrationd`.
- D-Bus `ListApps` and `RefreshApps`.
- JSON registry.

Acceptance:

- Installed Android packages appear in registry.

### Increment 4: Desktop Entries

Deliver:

- Desktop entry generator.
- Icon extraction.
- Stale entry cleanup.

Acceptance:

- Android app appears in launcher with correct name and icon.

### Increment 5: Launch Helper

Deliver:

- `droidianos-launch`.
- D-Bus `LaunchApp`.
- Lifecycle state updates.

Acceptance:

- Launcher starts Android app without showing Android home screen.

### Increment 6: APK Metadata Parser

Deliver:

- APK parser library.
- Permission classifier.
- CLI inspection tool.

Acceptance:

- Fixture APK metadata matches expected JSON.

### Increment 7: APK Installer Backend

Deliver:

- D-Bus `InspectApk`.
- D-Bus `InstallApk`.
- Transaction state.

Acceptance:

- APK installs from CLI and updates app registry.

### Increment 8: APK Installer UI

Deliver:

- Flutter installer screen.
- APK MIME registration.

Acceptance:

- Double-click APK opens installer and installs app.

### Increment 9: Software Center MVP

Deliver:

- Backend abstraction.
- Installed apps screen.
- APT and APK install/remove support.

Acceptance:

- Native and Android apps appear in one installed-apps list.

### Increment 10: Flatpak Backend

Deliver:

- Flatpak search/install/remove/update support.

Acceptance:

- Flatpak apps appear in same search and installed views.

### Increment 11: Permissions Service

Deliver:

- Permission registry.
- D-Bus `ListPermissions` and `SetPermission`.
- Basic Android permission sync.

Acceptance:

- User can view and toggle supported permissions.

### Increment 12: Settings App MVP

Deliver:

- App permissions page.
- Storage and shared folders page.
- Diagnostics page.

Acceptance:

- Per-app settings persist and apply.

### Increment 13: Notification Bridge

Deliver:

- Android notification listener bridge.
- Linux notification emission.
- Action mapping where supported.

Acceptance:

- Android app notifications appear in desktop notification center.

### Increment 14: Update Service

Deliver:

- Unified update service.
- APT update adapter.
- Flatpak update adapter.
- Android image update adapter.
- APK update placeholder.

Acceptance:

- Updates screen lists all supported update types.

### Increment 15: Security Hardening

Deliver:

- Shared-folder policy.
- Polkit rules.
- Permission prompts.
- Threat model review.

Acceptance:

- Default install does not expose full home directory to Android apps.

### Increment 16: ISO Build

Deliver:

- live-build configuration.
- Signed local repository.
- Bootable ISO.

Acceptance:

- ISO boots, installs, and passes smoke tests.

### Increment 17: Beta Stabilization

Deliver:

- Upgrade tests.
- Recovery tools.
- Diagnostics export.
- Performance profiling.

Acceptance:

- Beta image is usable on supported hardware and in VM.

### Increment 18: 1.0 Release

Deliver:

- Signed ISO.
- Release notes.
- Security notes.
- Upgrade path.
- Known issues.
- Release manifest.
- `SHA256SUMS` and optional detached signature.

Acceptance:

- All M10 criteria pass.

## Production Readiness Checklist

- Bootable ISO produced from clean CI.
- Signed packages and checksums.
- D-Bus interfaces versioned.
- Privileged operations protected by polkit.
- APK install path tested in VM.
- App launch path tested in VM.
- Update path tested from previous release.
- No user-facing Android subsystem terminology in normal flows.
- Recovery path documented.
- Security model reviewed.
