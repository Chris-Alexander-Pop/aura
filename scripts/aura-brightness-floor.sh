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
  meta="$(curl -sf "${BASE}/api/meta" 2>/dev/null || true)"
  if [[ -n "$meta" ]]; then
    token="${AURA_HTTP_TOKEN:-}"
    if [[ -z "$token" && -n "${XDG_RUNTIME_DIR:-}" && -r "${XDG_RUNTIME_DIR}/aura-http-token" ]]; then
      token="$(cat "${XDG_RUNTIME_DIR}/aura-http-token")"
    fi
    if [[ -z "$token" ]]; then
      token="$(python3 -c "import json,sys; print(json.load(sys.stdin).get('http_token',''))" <<<"$meta" 2>/dev/null || true)"
    fi
    curl -sf -X POST "${BASE}/api/Brightness.Set" \
      -H "Content-Type: application/json" \
      -H "X-Aura-Token: ${token}" \
      -d '{"monitor":"active","percent":0.01}' >/dev/null 2>&1 || true
    break
  fi
  sleep 0.2
done
