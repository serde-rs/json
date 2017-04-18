#!/bin/bash

set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

channel() {
    if [ -n "${TRAVIS}" ]; then
        if [ "${TRAVIS_RUST_VERSION}" = "${CHANNEL}" ]; then
            pwd
            (set -x; cargo "$@")
        fi
    elif [ -n "${APPVEYOR}" ]; then
        if [ "${APPVEYOR_RUST_CHANNEL}" = "${CHANNEL}" ]; then
            pwd
            (set -x; cargo "$@")
        fi
    else
        pwd
        (set -x; cargo "+${CHANNEL}" "$@")
    fi
}

if [ -n "${CLIPPY}" ]; then
    # cached installation will not work on a later nightly
    if [ -n "${TRAVIS}" ] && ! cargo install clippy --debug --force; then
        echo "COULD NOT COMPILE CLIPPY, IGNORING CLIPPY TESTS"
        exit
    fi

    cargo clippy -- -Dclippy
else
    CHANNEL=nightly
    channel clean
    channel build
    (cd "$DIR/tests/deps" && channel build)
    channel test
    channel test --features preserve_order

    for CHANNEL in stable 1.12.0 1.13.0 beta; do
        channel clean
        channel build
        channel build --features preserve_order
    done
fi
