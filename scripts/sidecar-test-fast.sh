#!/usr/bin/env bash
# Fast sidecar test gate: unit + fixture contracts + RPC manifest + HTTP server.
# Skips #[ignore] slow host sweeps (readonly_gap_methods_resolve_slow_host).
set -euo pipefail

if [[ -f "${HOME}/.cargo/env" ]]; then
  # shellcheck disable=SC1091
  source "${HOME}/.cargo/env"
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

"${ROOT}/scripts/generate-rpc-manifest.sh"
"${ROOT}/scripts/check-api-rpc-contract.sh"

cd "${ROOT}/sidecar"

cargo test --lib
cargo test --tests
