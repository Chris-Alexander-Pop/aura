# Greenfield stubs and platform backlog

**Status:** Implemented  
**Aligns with:** [BACKEND_TODO.md](../../BACKEND_TODO.md) §3.9–3.11, §3.14, §4 cross-cutting, §7 items 8+

---

## Goal

Capture **P3/P4 greenfield** and **platform** work without blocking P0–P2 completion. Prefer ADRs + manifest stubs over half-built services.

---

## Buckets

### Communication hub (§3.9)

- ADR: Matrix vs email vs unified inbox scope.
- Stub `Communication.*` RPCs return `not_implemented` until provider chosen.
- Keep existing [communication.rs](../../../sidecar/src/services/communication.rs) read-only depth.

### IDE / dev environment (§3.10)

- ADR: which editors (Cursor, VS Code, Neovim) and what RPCs mean.
- Defer implementation; link `docs/roadmap/`.

### Voice / exocortex (§3.11)

- ADR: local whisper vs cloud; privacy boundary.
- No sidecar code until model path defined.

### Gamemode / gaming overlay (§3.14)

- Review [gamemode.rs](../../../sidecar/src/services/gamemode.rs) vs gamescope; document what is live vs stub.
- UI: bar indicator only if `Gamemode.GetStatus` reliable.

### Cross-cutting §4

| Item | Action |
|------|--------|
| Observability | OpenTelemetry export — defer; document env hooks |
| Multi-user | Out of scope ADR |
| Plugin system | Defer |
| Theming pipeline | Partial in Settings; link GTK CSS vars doc |
| Backup/restore | Extend Settings export; vault track in [integrations_phase2.md](integrations_phase2.md) |
| i18n | Defer |
| Accessibility | React a11y pass separate from sidecar |

### Hyprland / compositor extras

- Document what stays in AGS TS vs sidecar `Hyprland.*`.
- Avoid duplicating dispatch in both layers.

---

## Deliverables

1. `docs/ARCHITECTURE_DECISIONS.md` entries for Communication, Voice, IDE (short)
2. BACKEND_TODO §3.9–3.11, §3.14: mark `[~]` with “ADR + defer”
3. Optional: trim dead RPCs from manifest for greenfield namespaces

---

## Acceptance

- No new failing tests from stubs (`not_implemented` returns JSON-RPC error, not panic)
- Roadmap links from BACKEND_TODO
- Team agrees P3/P4 not required for “backend complete” definition in [completion/README.md](README.md)

---

## Suggested order

ADRs first → manifest hygiene → gamemode doc review
