# P0 network and audio depth

**Status:** Planned  
**Depends on:** [p0_daily_hardening.md](../p0_daily_hardening.md), keyring patterns in network/vpn  
**Aligns with:** [BACKEND_TODO.md](../../BACKEND_TODO.md) §2.1–2.4, §7 item 5

---

## Goal

Close **daily-use** gaps: reliable `Network.Connect`, richer audio tests, optional BT pairing prep.

---

## Vertical slices

### Slice A — Network Connect hardening

- [network.rs](../../../sidecar/src/services/network.rs): WPA3/open paths; clear errors from nmcli.
- **802.1X:** document deferral or minimal EAP support if NM profiles exist.
- Captive portal: read-only `Network.GetCaptivePortal` or heuristic (HTTP 204 probe) — optional.
- Fixtures: `tests/fixtures/nmcli/connect_*.txt` + `integration_contracts.rs`.
- `#[ignore]` integration: connect to test SSID with `AURA_NETWORK_TEST_SSID` env.

**BACKEND_TODO:** §2.2 open items except roadmap-only.

### Slice B — Network tests

- Extend [network_rpc_shapes.rs](../../../sidecar/tests/network_rpc_shapes.rs).
- Unit tests for saved-connection parser edge cases.

### Slice C — Audio fixtures + profiles

- `tests/fixtures/audio/pactl_list_sinks.txt`, `wpctl_status.txt` (extend).
- `integration_contracts.rs` parse tests.
- Verify or gate `Audio.Effects.*` / `Audio.Profiles.*` behind `AURA_AUDIO_ADVANCED=1` if half-implemented.

**BACKEND_TODO:** §2.4 effects, profiles, pactl tests, volume round-trip `#[ignore]`.

### Slice D — Bluetooth (optional)

- Pairing agent: document “needs PIN UI” — no full implement unless roadmap demands.
- A2DP/HSP profile switch: research `bluetoothctl` commands; stub RPC if needed.

**BACKEND_TODO:** §2.3 pairing, audio profile — likely remain `[~]`.

### Slice E — Power fixtures

- Complete §2.1 sysfs fixture unit tests if any gaps remain in `integration_contracts.rs`.

---

## Test strategy

- No `Network.Connect` in default `call_method` (deny-list unchanged).
- Storage tests use temp DB only.

---

## Acceptance

- nmcli fixtures cover connect success/failure parse paths
- `./scripts/sidecar-test-fast.sh` green
- BACKEND_TODO §2.1–2.4: open P0 items `[x]` or `[~]` with reason

---

## Suggested order

A → B → C → E → D (optional)
