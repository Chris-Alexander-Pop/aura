# System debugging popup (roadmap)

## Summary

A **small, always-reachable** surface (popup or panel) for **triage and recovery** when things go wrong: **on-demand** diagnostics, **suggested** fixes, **log tail** of relevant services, and **paths to restore** from [vault-panel.md](./vault-panel.md) or **system** snapshots—without requiring the user to drop to a TTY first (though TTY instructions should be a **fallback**).

## Current baseline

[../feature_matrix.md](../feature_matrix.md) includes **logs** service UI not implemented; this popup may **orchestrate** that RPC rather than reimplementing log viewers.

## Goals

### Quick / autodebugging on request, restore system backups etc..

- **Autodebug**: run a **constrained** set of checks (network, DNS, pipewire, `systemd --user` failed units, last **OOM**, disk space, `dmesg` tail) and present a **plain-language** summary with **copyable** technical bundle for support.
- **Restore**: links/wizards to **restic/borg/backup** restore or **Timeshift**-class tools if present (read-only **detect** and **open**).
- **Safe mode**: e.g. “disable extensions” for Aura, **reset** `config` to last known good (TBD).

## Out of scope / risks

- **Auto-remediation** (restart `pipewire` without prompt) can disrupt work; use **consent** for destructive steps.
- **Log** collection may include **secrets**; **mask** and warn before **share**.

## Dependencies

- [`sidecar/src/services/logs.rs`](../../sidecar/src/services/logs.rs) and any future **diagnostics** RPC.
- [vault-panel.md](./vault-panel.md) and [system-and-input-foundation.md](./system-and-input-foundation.md) for **backup/restore** narrative.
- [lock-panel.md](./lock-panel.md) for **post-wake** issues and sleep.

## Open questions

1. Should **one-click** “send diagnostics” upload exist, or only **local copy**?
2. **Integration** with `journalctl` + **Hyprland** log for compositor-specific issues?
3. **Offline** when network is down: all checks still **local**?
