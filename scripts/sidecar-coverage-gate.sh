#!/usr/bin/env bash
# Fail when sidecar core line/region/function coverage is below the threshold (default 85%).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIN="${SIDECAR_COVERAGE_MIN:-85}"
unset SIDECAR_COVERAGE_IGNORE_REGEX

summary="$("${ROOT}/scripts/sidecar-coverage.sh" --summary-only 2>&1)" || {
  echo "${summary}" >&2
  exit 1
}

totals="$(printf '%s\n' "${summary}" | awk '/^TOTAL / { print; exit }')"
if [[ -z "${totals}" ]]; then
  echo "error: could not find TOTAL row in coverage summary" >&2
  printf '%s\n' "${summary}" >&2
  exit 1
fi

# TOTAL  regions missed  region%  functions missed  function%  lines missed  line%  ...
region="$(awk '{ gsub(/%/, "", $4); print $4 }' <<<"${totals}")"
function="$(awk '{ gsub(/%/, "", $7); print $7 }' <<<"${totals}")"
line="$(awk '{ gsub(/%/, "", $10); print $10 }' <<<"${totals}")"

echo "Sidecar coverage gate (minimum ${MIN}%):"
echo "  regions:   ${region}%"
echo "  functions: ${function}%"
echo "  lines:     ${line}%"

fail=0
for metric in region function line; do
  pct="${!metric}"
  if awk -v pct="${pct}" -v min="${MIN}" 'BEGIN { exit !(pct + 0 < min + 0) }'; then
    echo "error: ${metric} coverage ${pct}% is below ${MIN}%" >&2
    fail=1
  fi
done

if [[ "${fail}" -ne 0 ]]; then
  echo "Run ./scripts/sidecar-coverage.sh for HTML report (sidecar/target/coverage/html/)." >&2
  exit 1
fi

echo "Coverage gate passed."
