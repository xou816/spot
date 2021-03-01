#!/bin/bash

export SRC="$1"
export CARGO_TARGET_DIR="$2"/target
export APP_BIN="$3"
export OUTPUT="$4"
export BUILDTYPE="$5"
export FEATURES="$6"
export OFFLINE="$7"

echo $BUILDTYPE

if [[ $BUILDTYPE = "release" ]]; then
    OUTPUT_BIN="$CARGO_TARGET_DIR"/release/"$APP_BIN"
    PROFILE_ARG="--release"
else
    OUTPUT_BIN="$CARGO_TARGET_DIR"/debug/"$APP_BIN"
    PROFILE_ARG="--verbose"
fi

if [[ $OFFLINE = "true" ]]; then
    export CARGO_HOME="$SRC"/cargo

    cargo --offline build --features "$FEATURES" --manifest-path "$SRC"/Cargo.toml \
        "$PROFILE_ARG" && \
        cp "$OUTPUT_BIN" "$OUTPUT"
else
    cargo build --features "$FEATURES" --manifest-path "$SRC"/Cargo.toml \
        "$PROFILE_ARG" && \
        cp "$OUTPUT_BIN" "$OUTPUT"
fi

