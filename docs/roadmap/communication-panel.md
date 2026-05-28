# Communication panel (roadmap)

## Summary

A **unified communication** hub on Linux with a **Beeper**-like goal: **one inbox** for multiple chat networks (Matrix, Signal, WhatsApp via bridges where legal, SMS, email, etc.), with sane notifications and search. Full parity with commercial “all-in-one” apps is unlikely in v1; the roadmap prioritizes **pluggable backends** and a **Caelestia-consistent** UI in `ui/`, driven by sidecar or external daemons over stable APIs.

## Current baseline

Not a dedicated row in [../feature_matrix.md](../feature_matrix.md); may align with future “Communication” or general `ui/` routing. Caelestia legacy may have no direct equivalent—treat as **greenfield** with strong privacy review.

## Goals

### Beeper clone implemented in linux..

**Interpretation**: aggregate **multiple providers** into one thread list, with **per-account** muting, **unread** counts, and **quick reply** where APIs allow. **TBD**: which protocols are P1 (e.g. Matrix + email + IRC) vs later.

**Candidate scope**: 

- **Phase 1**: Read-only or one-way integrations for “hard” networks; full two-way only where FOSS bridges exist and the user opts in.
- **Phase 2**: **Embeds** for web-only services in a sandboxed webview (user risk acknowledgment).

## Out of scope / risks

- **TOS-violating** scraping of proprietary messengers is a legal and account-ban risk; prefer **official APIs** and **bridges** the user runs.
- **Notification flood** must align with [system-and-input-foundation.md](./system-and-input-foundation.md) notification fixes.

## Dependencies

- [top-dropdown.md](./top-dropdown.md) for **quickviews** of unread.
- Optional third-party bridges (e.g. `mautrix`, `signal-cli`) run as **user services**, not inside the shell binary.
- Keyring for OAuth and bridge tokens.

## Open questions

**Resolved (2026-05-28):** Unified comms hub is **out of scope** for Aura sidecar v1 — separate app/project with integration later. See [../ARCHITECTURE_DECISIONS.md](../ARCHITECTURE_DECISIONS.md) §13.
