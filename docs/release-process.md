# Release Process

Project packages are released through the project-owned signed APT repository.

Flow:

1. Build `.deb` packages in CI.
2. Run package and VM smoke tests.
3. Publish packages to a staged APT repository with `aptly` or `reprepro`.
4. Sign repository metadata.
5. Run `apt update` and upgrade tests against staging.
6. Build the ISO.
7. Run ISO boot and installed-system smoke tests.
8. Generate release artifacts with `sh ./build/scripts/release-artifacts.sh`.
9. Promote staging to stable.

Do not upload project packages directly to Debian's official archive for early releases.

Set `DROIDIANOS_GPG_KEY` before generating release artifacts to create a detached signature for `SHA256SUMS`.
