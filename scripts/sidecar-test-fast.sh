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

# CI: serialize rustc/link jobs. Parallel rust-lld has SIGBUS'd on GHA while
# linking many integration-test bins; the workflow also rewrites .cargo to bfd.
if [[ "${CI:-}" == "true" ]]; then
  export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
fi

cargo test --lib --no-fail-fast -- --test-threads=1
cargo test --tests --no-fail-fast -- --test-threads=1

"${ROOT}/scripts/sidecar-target-prune.sh"
