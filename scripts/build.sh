#! /usr/bin/env sh

SCRIPT_DIRECTORY="$(dirname -- "$(readlink -f -- "$0")")"
PROJECT_DIRECTORY="$(dirname -- "$SCRIPT_DIRECTORY")"
BUILD_DIRECTORY="$PROJECT_DIRECTORY"/build

NUMBER_OF_PROCESSORS="$(nproc)"

# Build and pass-through any options supplied. Default options are overridden if any options are supplied
if [ $# -eq 0 ];then
    set -o xtrace
    cargo build
    set +o xtrace
else
    set -o xtrace
    cargo build "$@"
    set +o xtrace
fi