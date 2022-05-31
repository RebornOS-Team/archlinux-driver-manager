#! /usr/bin/env sh

SCRIPT_DIRECTORY="$(dirname -- "$(readlink -f -- "$0")")"
PROJECT_DIRECTORY="$(dirname -- "$(dirname -- "$SCRIPT_DIRECTORY")")"

if ls "$SCRIPT_DIRECTORY"/*.pkg.tar.* > /dev/null 2>&1;then
    set -o xtrace
    rm "$SCRIPT_DIRECTORY"/*.pkg.tar.*
    set +o xtrace
fi

( # Create subshell to nullify directory changes on exit
    # Run makepkg
    set -o xtrace
    cd "$SCRIPT_DIRECTORY" && \
    makepkg \
        --force \
        --syncdeps \
        "$@"
    set +o xtrace
)