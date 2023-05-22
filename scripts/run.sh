#! /usr/bin/env sh

SCRIPT_DIRECTORY="$(dirname -- "$(readlink -f -- "$0")")"
PROJECT_DIRECTORY="$(dirname -- "$SCRIPT_DIRECTORY")"
BUILD_DIRECTORY="$PROJECT_DIRECTORY"/build

# Run build.sh if the build directory does not exist
if [ ! -d "$BUILD_DIRECTORY" ];then 
    sh "$SCRIPT_DIRECTORY"/build.sh
fi

# Run and pass-through any options supplied. Default options are overridden if any options are supplied
if [ $# -eq 0 ];then
    set -o xtrace
    cargo run
    set +o xtrace
else
    set -o xtrace
    cargo run -- "$@"
    set +o xtrace
fi