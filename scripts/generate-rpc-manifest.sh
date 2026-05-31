#!/usr/bin/env bash
# Generate sidecar/rpc-manifest.json from registry.register calls.
# Security.Offensive.* is included only when AURA_OFFENSIVE_MANIFEST=1 (matches default build).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/sidecar/rpc-manifest.json"
TMP="$(mktemp)"
rg -o 'registry\.register\("([^"]+)"' "$ROOT/sidecar/src/services" --no-filename \
  | sed 's/registry.register("//;s/"$//' \
  | sort -u \
  | { if [[ "${AURA_OFFENSIVE_MANIFEST:-0}" == 1 ]]; then cat; else rg -v '^Security\.Offensive\.' || true; fi; } \
  | jq -R -s 'split("\n") | map(select(length > 0))' > "$TMP"
mv "$TMP" "$OUT"
echo "Wrote $OUT ($(jq 'length' "$OUT") methods)"
