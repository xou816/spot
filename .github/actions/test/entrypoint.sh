#!/bin/sh
meson target
cargo fmt -- --check && \
	meson install -C target && \
	meson test -C target