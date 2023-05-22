# Archlinux Driver Manager

A tool to detect and install drivers for hardware on Arch Linux.

[![Discord Server](https://dcbadge.vercel.app/api/server/cU5s6MPpQH?style=flat)](https://discord.gg/cU5s6MPpQH)
[![License: MPL v2.0](https://img.shields.io/badge/License-MPL--2.0-blue.svg)](https://www.mozilla.org/en-US/MPL/2.0/)
![GitHub release (latest by date)](https://img.shields.io/github/v/release/rebornos-developers/archlinux-driver-manager)
[![Release](https://github.com/RebornOS-Developers/archlinux-driver-manager/actions/workflows/release.yml/badge.svg)](https://github.com/RebornOS-Developers/archlinux-driver-manager/actions/workflows/release.yml)
[![Pre-Release (Git)](https://github.com/RebornOS-Developers/archlinux-driver-manager/actions/workflows/pre_release.yml/badge.svg)](https://github.com/RebornOS-Developers/archlinux-driver-manager/actions/workflows/pre_release.yml)

> **Note**: This project should not carry any RebornOS-specific configuration except for the application packaging files (PKGBUILD, build scripts), icons, and launch scripts. Use the [archlinux-driver-manager-db](https://github.com/RebornOS-Developers/archlinux-driver-manager-db) project for other RebornOS-specific configuration.

## Cloning

In order to download the source code to your local computer for testing, or for development, you can clone from the remote repository using either SSH, or HTTPS. Below are instructions on how to do so using Gitlab hosted code as remote.

### HTTPS

```bash
git clone https://github.com/RebornOS-Developers/archlinux-driver-manager.git 
```

OR

### SSH

```bash
git clone git@github.com:RebornOS-Developers/archlinux-driver-manager.git
```

## Local development

### 1. Build

The below script will build the program (and install any prerequisites). Change to the project directory (`cd archlinux-driver-manager`) and run the below. You can specify any commandline parameters to `cargo build` by passing it to the below script

```bash
sh scripts/build.sh
```

### 2. Run
Change to the project directory (`cd archlinux-driver-manager`) and run the below. You can specify any commandline parameters to `archlinux-driver-manager` by passing it to the below script

```bash
sh scripts/run.sh
```

## Packaging

Change to the project directory (`cd archlinux-driver-manager`) and run any of the below scripts:
- `sh packaging/setup.sh <MODE>`: Builds and installs a package
- `sh packaging/build-package.sh <MODE>`: Just builds a package without installing it locally
- `sh packaging/install-package.sh <MODE>`: Just installs a package locally, except if no built package is detected, a package is built.

- where `<MODE>` can be one of the below
     1. `local`: Selects *archlinux-driver-manager-local* from the local project that you have cloned already.
     2. `git`: Selects *archlinux-driver-manager-git* from the latest git commit.
     3. `stable`: Selects *archlinux-driver-manager* from the git tag corresponding to the [`pkgver` specified in the PKGBUILD](https://github.com/RebornOS-Developers/archlinux-driver-manager/blob/main/packaging/archlinux-driver-manager/PKGBUILD#L5). If `pkgver=0.0.1`, then the git tag `v0.0.1` is used for packaging. 
     
> **Note**: Any additional parameters passed to the above scripts are automatically sent to `makepkg` or `pacman` (whichever is applicable).
