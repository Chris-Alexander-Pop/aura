#!/usr/bin/env bash
# Fast sidecar test gate: unit + fixture contracts + RPC manifest + HTTP server.
# Skips #[ignore] slow host sweeps (readonly_gap_methods_resolve_slow_host).
set -euo pipefail

if [[ -f "${HOME}/.cargo/env" ]]; then
  # shellcheck disable=SC1091
  source "${HOME}/.cargo/env"
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck disable=SC1091
source "${ROOT}/scripts/rust-cache-env.sh"

"${ROOT}/scripts/generate-rpc-manifest.sh"
"${ROOT}/scripts/generate-openapi.sh"
"${ROOT}/scripts/check-api-rpc-contract.sh"

cd "${ROOT}/sidecar"

# CI: avoid rustc 1.98 rust-lld SIGBUS and runner disk pressure while linking
# many integration-test binaries. Local Arch keeps LLD via
# sidecar/.cargo/config.toml unless the caller overrides these.
if [[ "${CI:-}" == "true" ]]; then
  export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
  # Replaces target rustflags from sidecar/.cargo/config.toml (fuse-ld=lld).
  # link-self-contained=off alone still leaves rustc requesting -fuse-ld=lld
  # without apt lld installed; pair it with explicit system bfd.
  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="${CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS:--C link-self-contained=off -C link-arg=-fuse-ld=bfd}"
fi

cargo test --lib --no-fail-fast -- --test-threads=1

# Drop stale test artifacts before compiling the large integration-test set.
# Concurrent GHA runners have hit "No space left on device" during this phase.
if [[ "${CI:-}" == "true" ]]; then
  "${ROOT}/scripts/sidecar-target-prune.sh" --force || true
fi

cargo test --tests --no-fail-fast -- --test-threads=1

"${ROOT}/scripts/sidecar-target-prune.sh"
