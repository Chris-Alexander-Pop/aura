#!/usr/bin/env bash
# Generate sidecar/rpc-manifest.json from registry.register calls.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/sidecar/rpc-manifest.json"
TMP="$(mktemp)"
rg -o 'registry\.register\("([^"]+)"' "$ROOT/sidecar/src/services" --no-filename \
  | sed 's/registry.register("//;s/"$//' \
  | sort -u \
  | jq -R -s 'split("\n") | map(select(length > 0))' > "$TMP"
mv "$TMP" "$OUT"
echo "Wrote $OUT ($(jq 'length' "$OUT") methods)"
"${ROOT}/scripts/generate-openapi.sh"
