#!/usr/bin/env bash
# Aura media/brightness keys — sidecar mutations + OSD feedback.
set -euo pipefail

ACTION="${1:-}"
BASE="${AURA_SIDECAR_URL:-http://127.0.0.1:9080}"

api_get() {
  local method="$1"
  curl -sf "${BASE}/api/${method}" 2>/dev/null || echo '{"ok":false}'
}

api_post() {
  local method="$1"
  local body="${2:-{}}"
  curl -sf -X POST "${BASE}/api/${method}" \
    -H "Content-Type: application/json" \
    -d "$body" 2>/dev/null || echo '{"ok":false}'
}

json_float() {
  python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('data',{}).get('$1',0))" 2>/dev/null || echo "0"
}

json_int() {
  python3 -c "import json,sys; d=json.load(sys.stdin); print(int(d.get('data',{}).get('$1',0)))" 2>/dev/null || echo "0"
}

default_sink_id() {
  api_get "Audio.GetDevices" | python3 -c "
import json,sys
d=json.load(sys.stdin).get('data',{})
sinks=d.get('sinks') or []
for s in sinks:
  if s.get('is_default'):
    print(s.get('id',0)); exit()
if sinks: print(sinks[0].get('id',0))
" 2>/dev/null || echo "0"
}

default_source_id() {
  api_get "Audio.GetDevices" | python3 -c "
import json,sys
d=json.load(sys.stdin).get('data',{})
srcs=d.get('sources') or []
for s in srcs:
  if s.get('is_default'):
    print(s.get('id',0)); exit()
if srcs: print(srcs[0].get('id',0))
" 2>/dev/null || echo "0"
}

sink_volume() {
  api_get "Audio.GetDevices" | python3 -c "
import json,sys
d=json.load(sys.stdin).get('data',{})
for s in (d.get('sinks') or []):
  if s.get('is_default'):
    print(s.get('volume',0.5)); exit()
" 2>/dev/null || echo "0.5"
}

sink_muted() {
  api_get "Audio.GetDevices" | python3 -c "
import json,sys
d=json.load(sys.stdin).get('data',{})
for s in (d.get('sinks') or []):
  if s.get('is_default'):
    print('1' if s.get('muted') else '0'); exit()
print('0')
" 2>/dev/null || echo "0"
}

brightness_active() {
  api_post "Brightness.Get" '{"monitor":"active"}' | python3 -c "
import json,sys
d=json.load(sys.stdin).get('data',{})
print(d.get('brightness',0.5))
" 2>/dev/null || echo "0.5"
}

show_osd() {
  ags request osd "$1" 2>/dev/null || true
}

case "$ACTION" in
  volume-up)
    id="$(default_sink_id)"
    vol="$(sink_volume)"
    new="$(python3 -c "print(min(1.0, float('$vol')+0.05))")"
    api_post "Audio.SetSinkVolume" "{\"device_id\":$id,\"volume\":$new}" >/dev/null
    show_osd volume
    ;;
  volume-down)
    id="$(default_sink_id)"
    vol="$(sink_volume)"
    new="$(python3 -c "print(max(0.0, float('$vol')-0.05))")"
    api_post "Audio.SetSinkVolume" "{\"device_id\":$id,\"volume\":$new}" >/dev/null
    show_osd volume
    ;;
  mute)
    id="$(default_sink_id)"
    muted="$(sink_muted)"
    next="$([ "$muted" = "1" ] && echo false || echo true)"
    api_post "Audio.SetSinkMute" "{\"device_id\":$id,\"muted\":$next}" >/dev/null
    show_osd volume
    ;;
  mic-mute)
    id="$(default_source_id)"
    muted="$(api_get "Audio.GetDevices" | python3 -c "
import json,sys
d=json.load(sys.stdin).get('data',{})
for s in (d.get('sources') or []):
  if s.get('is_default'):
    print('1' if s.get('muted') else '0'); exit()
print('0')
")"
    next="$([ "$muted" = "1" ] && echo false || echo true)"
    api_post "Audio.SetSourceMute" "{\"device_id\":$id,\"muted\":$next}" >/dev/null
    show_osd mic
    ;;
  brightness-up)
    b="$(brightness_active)"
    new="$(python3 -c "print(min(1.0, float('$b')+0.05))")"
    api_post "Brightness.Set" "{\"monitor\":\"active\",\"percent\":$new}" >/dev/null
    show_osd brightness
    ;;
  brightness-down)
    b="$(brightness_active)"
    new="$(python3 -c "print(max(0.05, float('$b')-0.05))")"
    api_post "Brightness.Set" "{\"monitor\":\"active\",\"percent\":$new}" >/dev/null
    show_osd brightness
    ;;
  *)
    echo "usage: aura-media-key.sh {volume-up|volume-down|mute|mic-mute|brightness-up|brightness-down}" >&2
    exit 1
    ;;
esac
