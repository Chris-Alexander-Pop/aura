# P0 daily-use hardening (Power, Network, Audio, Brightness, System)

**Status:** Implemented (2026-05-31)  
**Depends on:** [control_center_foundation.md](control_center_foundation.md), [foundation_contracts.md](foundation_contracts.md) (fixtures)  
**Aligns with:** [BACKEND_TODO.md](../BACKEND_TODO.md) §2.1–2.6, §2.5, §6.1

---

## Goal

Harden **daily shell RPCs** with real parsers, tests, and safe mutations:

1. **Power** — sysfs fixture unit tests; profile integration with mocked `powerprofilesctl` output.
2. **Network** — `nmcli` fixtures; `Network.Connect` for WPA (keyring); 802.1X deferred; captive portal helper optional.
3. **Audio** — `pactl`/`wpctl` fixtures; volume round-trip behind `#[ignore]` only.
4. **Brightness** — multi-monitor `Brightness.Get`/`Set`, wire parser to `brightnessctl`/`ddcutil`.
5. **System** — `System.GetStats` CPU/RAM/GPU/temps via `sysinfo` + sensors; satisfy P0 smoke.

---

## Non-goals

| Item | Reason |
|------|--------|
| VPN | [vpn_service.md](vpn_service.md) |
| Bluetooth pairing UI | PIN/agent deferred in BACKEND_TODO |
| Performance pane depth | [control_center_hardening.md](control_center_hardening.md) |

---

## Test strategy (current layout)

| Service | Read-only shapes | Fixtures / unit | Mutations |
|---------|------------------|-----------------|-----------|
| Power | `power_rpc_shapes.rs` | `tests/fixtures/power_supply/*` + `integration_contracts.rs` | `Power.SetProfile` denied |
| Network | `network_rpc_shapes.rs` | `tests/fixtures/nmcli/*` | `Network.Connect` denied; optional `#[ignore]` connect in isolated test |
| Audio | `audio_rpc_shapes.rs` | `tests/fixtures/audio/*` (extend) | All `Audio.Set*` + `Audio.Media.*` denied |
| System | `system_rpc_shapes.rs` | Mock sysinfo thresholds in `#[cfg(test)]` | — |
| Brightness | Add `brightness_rpc_shapes.rs` when RPC stable | Parser tests in `brightness.rs` `#[cfg(test)]` | `Brightness.Set` denied |

**Integration:**

- Add completed methods to `P0_METHODS` in `integration_test.rs` only when shape-stable.
- Slow host tools → `READONLY_GAP_SLOW_HOST`, not fast gate.

---

## Vertical slices

### Slice A — Fixture parsers (Power, Network, Audio) — **P0**

Unit tests in `integration_contracts.rs` or service `#[cfg(test)]` using `tests/fixtures/`.

### Slice B — `System.GetStats` — **P1**

Implement real stats; extend `system_rpc_shapes.rs` with field asserts (`cpu_percent`, `memory`, etc.).

### Slice C — Brightness OSD path — **P1**

`Brightness.Get`/`Set` multi-monitor; GTK/React OSD consumers in `api.ts`.

### Slice D — `Network.Connect` (WPA) — **P0**

Keyring + nmcli; integration test **only** under `#[ignore]` with documented test SSID or mock.

---

## Suggested order

A → B → C → D

---

## Acceptance

- P0 smoke: `Audio.GetDevices`, `System.GetStats`, `Brightness.Set` (mock name) in `p0_methods_resolve_without_panic`  
- No new deny-list violations in default `call_method` tests  
- Fixture tests pass without Wi-Fi/PipeWire hardware
