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

# Cap parallel rustc/link jobs. Unlimited parallelism + rust-lld has
# SIGBUS'd on GitHub Actions while linking many integration test bins.
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"

if [[ "${CI:-}" == "true" ]]; then
  # Replace sidecar/.cargo fuse-ld=lld (empty env does not clear config.toml).
  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="${CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS:--C link-arg=-fuse-ld=bfd}"
  export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
  export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
  export CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG:-0}"
  df -h || true
fi

cargo test --lib --no-fail-fast -- --test-threads=1

if [[ "${CI:-}" == "true" ]]; then
  # Drop incremental/scratch before linking the large --tests set.
  rm -rf target/debug/incremental
  AURA_TARGET_MAX_GB=1 "${ROOT}/scripts/sidecar-target-prune.sh" --force || true
  df -h || true
fi

cargo test --tests --no-fail-fast -- --test-threads=1

"${ROOT}/scripts/sidecar-target-prune.sh"
