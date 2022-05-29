#! /usr/bin/env sh

SCRIPT_DIRECTORY="$(dirname -- "$(readlink -f -- "$0")")"
PROJECT_DIRECTORY="$(dirname -- "$(dirname -- "$SCRIPT_DIRECTORY")")"

archlinux-driver-manager "$@"