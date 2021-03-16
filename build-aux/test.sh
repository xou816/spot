#!/bin/bash

export SRC="$1"
export CARGO_TARGET_DIR="$2"/target
export BUILDTYPE="$3"
export OFFLINE="$4"

echo "$BUILDTYPE"

if [[ $BUILDTYPE = "release" ]]; then
    PROFILE_ARG="--release"
else
    PROFILE_ARG="--verbose"
fi

if [[ $OFFLINE = "true" ]]; then
    export CARGO_HOME="$SRC"/cargo
fi

cargo test --manifest-path "$SRC"/Cargo.toml "$PROFILE_ARG"