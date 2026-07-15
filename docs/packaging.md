# Packaging

Each package under `packages/` is a standalone Debian source package.

Build all packages with:

```sh
sh ./build/scripts/build-packages.sh
```

Build local repository and ISO with:

```sh
sh ./build/scripts/build-local-repo.sh
sh ./build/scripts/build-iso.sh
```
