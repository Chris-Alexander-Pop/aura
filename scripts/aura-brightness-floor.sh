#!/usr/bin/env bash
# Ensure laptop backlight is at least 1% at session start (0% turns panel off on some hardware).
set -euo pipefail

if ! command -v brightnessctl >/dev/null 2>&1; then
  exit 0
fi

device=""
if machine="$(brightnessctl -m 2>/dev/null | head -1)"; then
  device="${machine%%,*}"
fi
if [[ -z "$device" ]]; then
  device="intel_backlight"
fi

current="$(brightnessctl -d "$device" -m 2>/dev/null | cut -d, -f4 | tr -d '%' || echo 100)"
pct="${current%%.*}"
if [[ -z "$pct" ]] || [[ "$pct" -ge 1 ]]; then
  exit 0
fi

brightnessctl -d "$device" s 1% >/dev/null 2>&1 || true

BASE="${AURA_SIDECAR_URL:-http://127.0.0.1:9080}"
for _ in $(seq 1 30); do
  if curl -sf "${BASE}/api/meta" >/dev/null 2>&1; then
    curl -sf -X POST "${BASE}/api/Brightness.Set" \
      -H "Content-Type: application/json" \
      -d '{"monitor":"active","percent":0.01}' >/dev/null 2>&1 || true
    break
  fi
  sleep 0.2
done
