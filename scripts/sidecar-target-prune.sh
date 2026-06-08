#!/usr/bin/env bash
# Keep sidecar/target/ from ballooning (duplicate test binaries + llvm-cov trees).
#
# Env:
#   AURA_TARGET_MAX_GB       Prune aggressively when target/ exceeds this (default: 20)
#   AURA_TARGET_PRUNE_DRY_RUN  Set 1 to print removals without deleting
#
# Usage:
#   ./scripts/sidecar-target-prune.sh              # coverage cleanup + prune if over limit
#   ./scripts/sidecar-target-prune.sh --coverage-only
#   ./scripts/sidecar-target-prune.sh --force      # always dedupe stale test artifacts
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${ROOT}/sidecar/target"
MAX_GB="${AURA_TARGET_MAX_GB:-20}"
DRY_RUN="${AURA_TARGET_PRUNE_DRY_RUN:-0}"

MODE="${1:-}"

bytes_to_gb() {
  awk -v b="${1:-0}" 'BEGIN { printf "%.1f", b / 1024 / 1024 / 1024 }'
}

target_bytes() {
  if [[ -d "${TARGET}" ]]; then
    du -sb "${TARGET}" 2>/dev/null | cut -f1
  else
    echo 0
  fi
}

remove_path() {
  local path="$1"
  [[ -e "${path}" ]] || return 0
  if [[ "${DRY_RUN}" == "1" ]]; then
    echo "would remove ${path} ($(du -sh "${path}" 2>/dev/null | cut -f1))"
  else
    rm -rf "${path}"
    echo "removed ${path}"
  fi
}

prune_coverage_build_trees() {
  # Reports live under target/coverage/; rebuild trees are disposable.
  remove_path "${TARGET}/llvm-cov/debug"
  remove_path "${TARGET}/llvm-cov/llvm-cov-target"
  remove_path "${TARGET}/llvm-cov-target"
}

prune_stale_test_artifacts() {
  local deps="${TARGET}/debug/deps"
  [[ -d "${deps}" ]] || return 0

  local removed=0
  local -A keep_hash=()

  while IFS= read -r -d '' exe; do
    local base hash key mtime
    base="$(basename "${exe}")"
    [[ "${base}" =~ ^(.+)-([0-9a-f]{16})$ ]] || continue
    key="${BASH_REMATCH[1]}"
    hash="${BASH_REMATCH[2]}"
    mtime="$(stat -c '%Y' "${exe}")"

    if [[ -z "${keep_hash[${key}]+x}" ]]; then
      keep_hash["${key}"]="${hash}:${mtime}"
      continue
    fi

    local kept_hash kept_mtime
    kept_hash="${keep_hash[${key}]%%:*}"
    kept_mtime="${keep_hash[${key}]#*:}"

    if (( mtime > kept_mtime )); then
      # Drop the older executable hash and its sibling artifacts.
      while IFS= read -r -d '' artifact; do
        remove_path "${artifact}"
        removed=$((removed + 1))
      done < <(find "${deps}" -maxdepth 1 -name "*${kept_hash}*" -print0 2>/dev/null)
      keep_hash["${key}"]="${hash}:${mtime}"
    else
      while IFS= read -r -d '' artifact; do
        remove_path "${artifact}"
        removed=$((removed + 1))
      done < <(find "${deps}" -maxdepth 1 -name "*${hash}*" -print0 2>/dev/null)
    fi
  done < <(
    find "${deps}" -maxdepth 1 -type f -executable ! -name '*.so' -print0 2>/dev/null
  )

  if (( removed > 0 )); then
    echo "pruned stale test artifact groups: ${removed}"
  fi
}

run_cargo_sweep() {
  if ! command -v cargo-sweep >/dev/null 2>&1; then
    return 0
  fi
  echo "running cargo sweep (standby + older than 14 days)..."
  if [[ "${DRY_RUN}" == "1" ]]; then
    echo "would run: (cd sidecar && cargo sweep -s -t 14)"
    return 0
  fi
  (cd "${ROOT}/sidecar" && cargo sweep -s -t 14)
}

before_bytes="$(target_bytes)"
before_gb="$(bytes_to_gb "${before_bytes}")"

prune_coverage_build_trees

if [[ "${MODE}" == "--coverage-only" ]]; then
  after_gb="$(bytes_to_gb "$(target_bytes)")"
  echo "sidecar/target: ${before_gb} GB -> ${after_gb} GB (coverage build trees only)"
  exit 0
fi

max_bytes=$((MAX_GB * 1024 * 1024 * 1024))
force=0
[[ "${MODE}" == "--force" ]] && force=1

if (( force == 1 )) || (( before_bytes > max_bytes )); then
  echo "sidecar/target is ${before_gb} GB (limit ${MAX_GB} GB); pruning stale test artifacts..."
  prune_stale_test_artifacts
  run_cargo_sweep
fi

after_gb="$(bytes_to_gb "$(target_bytes)")"
echo "sidecar/target: ${before_gb} GB -> ${after_gb} GB"
