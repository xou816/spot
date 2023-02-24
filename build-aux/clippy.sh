#!/bin/bash

export SRC="$1"
export CARGO_TARGET_DIR="$2"/target
export OFFLINE="$3"

if [[ $OFFLINE = "true" ]]; then
    export CARGO_HOME="$SRC"/cargo
fi

cargo clippy --manifest-path "$SRC"/Cargo.toml -- -D warnings \
-A clippy::module_inception \
-A clippy::new_without_default \
-A clippy::enum-variant-names \
-A clippy::uninlined_format_args #tmp
