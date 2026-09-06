#!/usr/bin/env bash
# Source from sidecar test/coverage scripts. Enables sccache when installed.
# Install: pacman -S sccache  (or: cargo install sccache)

if command -v sccache >/dev/null 2>&1; then
  export RUSTC_WRAPPER=sccache
  export SCCACHE_DIR="${SCCACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/sccache}"
  mkdir -p "${SCCACHE_DIR}"
fi
