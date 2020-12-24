#!/bin/sh

export MESON_BUILD_ROOT="$1"
export MESON_SOURCE_ROOT="$2"
export CARGO_TARGET_DIR="$MESON_BUILD_ROOT"/target
export CARGO_HOME="$MESON_SOURCE_ROOT"/cargo

cargo test --manifest-path "$MESON_SOURCE_ROOT"/Cargo.toml
