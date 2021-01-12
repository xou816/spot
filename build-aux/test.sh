#!/bin/sh

export SRC="$1"
export OFFLINE="$2"

if [[ $OFFLINE = "true" ]]; then
    export CARGO_HOME="$SRC"/cargo
fi

cargo test --manifest-path "$SRC"/Cargo.toml