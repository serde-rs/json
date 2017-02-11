#!/bin/bash

set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

channel() {
    if [ -n "${TRAVIS}" ]; then
        if [ "${TRAVIS_RUST_VERSION}" = "${CHANNEL}" ]; then
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

    cd "$DIR/json"
    cargo clippy

    cd "$DIR/json_tests"
    cargo clippy
else
    CHANNEL=nightly
    cd "$DIR/json"
    channel clean
    channel build
    channel build --features preserve_order
    channel test
    cd "$DIR/json_tests"
    channel test
    channel test --features arbitrary_precision

    for CHANNEL in stable 1.12.0 1.13.0 beta; do
        cd "$DIR/json"
        channel clean
        channel build
        channel build --features preserve_order
    done
fi
