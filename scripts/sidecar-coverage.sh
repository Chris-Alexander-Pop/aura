#!/usr/bin/env bash
# Run ags-sidecar unit + integration tests with LLVM source coverage.
set -euo pipefail

if [[ -f "${HOME}/.cargo/env" ]]; then
  # shellcheck disable=SC1091
  source "${HOME}/.cargo/env"
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck disable=SC1091
source "${ROOT}/scripts/rust-cache-env.sh"
SIDECAR="${ROOT}/sidecar"
OUT_DIR="${SIDECAR}/target/coverage"
# Isolated target avoids stale/corrupt default llvm-cov-target (see sidecar/README.md).
export CARGO_TARGET_DIR="${SIDECAR}/target/llvm-cov"

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "error: cargo-llvm-cov not found." >&2
  echo "  rustup component add llvm-tools-preview" >&2
  echo "  cargo install cargo-llvm-cov" >&2
  exit 1
fi

mkdir -p "${OUT_DIR}/html"

cd "${SIDECAR}"

IGNORE_ARGS=()
if [[ -n "${SIDECAR_COVERAGE_IGNORE_REGEX:-}" ]]; then
  IGNORE_ARGS=(--ignore-filename-regex "${SIDECAR_COVERAGE_IGNORE_REGEX}")
fi

if [[ "${1:-}" == "--summary-only" ]]; then
  shift
  cargo llvm-cov --all-features --summary-only "${IGNORE_ARGS[@]}" -- --test-threads=1 "$@"
  exit 0
fi

# Extra args (e.g. --open) pass through after --
if [[ "${1:-}" == "--" ]]; then
  shift
fi

# --output-path is only valid with --lcov|--json|--text; --html uses --output-dir.
cargo llvm-cov \
  --all-features \
  --lcov \
  --output-path "${OUT_DIR}/lcov.info" \
  "${IGNORE_ARGS[@]}" \
  -- --test-threads=1 \
  "$@"

cargo llvm-cov report \
  --html \
  --output-dir "${OUT_DIR}" \
  "${IGNORE_ARGS[@]}" \
  "$@"

HTML_INDEX="${OUT_DIR}/html/index.html"
if [[ ! -f "${HTML_INDEX}" && -f "${OUT_DIR}/html/html/index.html" ]]; then
  HTML_INDEX="${OUT_DIR}/html/html/index.html"
fi

echo ""
echo "HTML report: file://${HTML_INDEX}"
echo "LCOV:        ${OUT_DIR}/lcov.info"

"${ROOT}/scripts/sidecar-target-prune.sh" --coverage-only
