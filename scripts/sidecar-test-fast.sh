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

# CI: avoid rustc 1.98 rust-lld SIGBUS and runner disk exhaustion while
# linking many large integration-test binaries. The workflow rewrites
# sidecar/.cargo to system bfd; keep these as defense in depth.
if [[ "${CI:-}" == "true" ]]; then
  export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
  export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
  export CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG:-0}"
  df -h || true
fi

cargo test --lib --no-fail-fast -- --test-threads=1

if [[ "${CI:-}" == "true" ]]; then
  # Reclaim debug artifacts before linking dozens of integration test bins.
  rm -rf target/debug/incremental || true
  if [[ -x "${ROOT}/scripts/sidecar-target-prune.sh" ]]; then
    AURA_TARGET_MAX_GB=1 "${ROOT}/scripts/sidecar-target-prune.sh" --force || true
  fi
  df -h || true
fi

cargo test --tests --no-fail-fast -- --test-threads=1

"${ROOT}/scripts/sidecar-target-prune.sh"
