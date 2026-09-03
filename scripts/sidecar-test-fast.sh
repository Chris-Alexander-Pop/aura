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

# GitHub Actions + rustc 1.98 rust-lld has SIGBUS'd when many large
# integration-test binaries link concurrently. Also keep peak disk low:
# ENOSPC has aborted ld.bfd after switching away from rust-lld.
if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
  export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
  export RUSTFLAGS="${RUSTFLAGS:--C linker-features=-lld -C debuginfo=line-tables-only}"
fi

cargo test --lib --no-fail-fast -- --test-threads=1

# Drop incremental + leftover unit-test bins before linking integration tests.
rm -rf target/debug/incremental || true
find target/debug -maxdepth 1 -type f -name 'ags_sidecar-*' -delete 2>/dev/null || true

cargo test --tests --no-fail-fast -- --test-threads=1

"${ROOT}/scripts/sidecar-target-prune.sh"
