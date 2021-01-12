#!/bin/bash

export SRC="$1"
export CARGO_TARGET_DIR="$2"/target
export OFFLINE="$3"

if [[ $OFFLINE = "true" ]]; then
    export CARGO_HOME="$SRC"/cargo
fi

cargo test --manifest-path "$SRC"/Cargo.toml