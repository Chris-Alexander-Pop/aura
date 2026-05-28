#!/usr/bin/env bash
# Fail if ui/src/lib/api.ts calls RPC methods missing from sidecar/rpc-manifest.json
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/sidecar/rpc-manifest.json"
API="$ROOT/ui/src/lib/api.ts"

if [[ ! -f "$MANIFEST" ]]; then
  echo "Missing $MANIFEST — run ./scripts/generate-rpc-manifest.sh first"
  exit 1
fi

mapfile -t API_METHODS < <(
  rg -o 'call(?:Data)?\("([^"]+)"' "$API" --no-filename \
    | sed 's/callData("//;s/call("//;s/"$//' \
    | sort -u
)

missing=0
for method in "${API_METHODS[@]}"; do
  if ! jq -e --arg m "$method" 'index($m) != null' "$MANIFEST" >/dev/null; then
    echo "MISSING in sidecar: $method"
    missing=1
  fi
done

if [[ "$missing" -ne 0 ]]; then
  exit 1
fi

echo "OK: ${#API_METHODS[@]} api.ts methods present in rpc-manifest.json"
