# Calendar panel (roadmap)

## Summary

A **full calendar experience** inside Aura: visual month/week/agenda, rich enough to compare to **Google Calendar** or **Notion**-style scheduling, with **sync** to other apps, optional **Reclaim.ai**-class automatic time blocking, **fitness/health** hooks, and a **todo** subpanel for fast capture and reminders.

Backend pieces exist in the sidecar ([`calendar.rs`](../../sidecar/src/services/calendar.rs), [`fitness.rs`](../../sidecar/src/services/fitness.rs) per [../feature_matrix.md](../feature_matrix.md)); the **React UI** is the main build effort. This panel likely loads in a **calendar** webview window (see `ags msg` window names in [AGENTS.md](../../AGENTS.md)).

## Current baseline

[../feature_matrix.md](../feature_matrix.md): **Calendar** and **Fitness** backends are “backend ready,” UI not implemented. Legacy Caelestia had a `calendar/` module as reference.

## Goals

### GCal/Notion+ style calendar within linux

**Polished** scheduling UI: multiple calendars, color coding, drag to reschedule (where provider allows), search, and **time zone** clarity. Target is “daily driver” quality, not a read-only agenda.

### Syncing between other applications

**Bidirectional or one-way** sync with major providers (Google, CalDAV, Microsoft) and optional **ICS** import/export. Conflict policy and **last-write-wins** vs prompt must be explicit.

### ReclaimAI+ features?

**Reclaim.ai** automates buffer blocks, focus time, and habit scheduling. **Candidate scope**: smart **suggestions** for focus blocks, travel time, and “preparation” events based on rules the user controls—without requiring a proprietary Reclaim backend. Mark **TBD** for how much ML vs heuristics.

### Fitness + health tracking / integration features

Link calendar to **workouts**, **sleep blocks**, or **fasting** windows—pulling from fitness service in sidecar or external APIs (Strava, Health Connect, etc.) with **strict privacy** controls.

### Todo subpanel (make it super easy to add stuff, sync across other apps, auto reminders)

**Inline task capture** with natural date parsing (“tomorrow 3pm”), **sync** to task systems (Todoist, local files, CalDAV VTODO), and **reminders** that surface in [control-panel.md](./control-panel.md) notifications and [system-and-input-foundation.md](./system-and-input-foundation.md) notification fixes.

## Out of scope / risks

- **Google Calendar API** compliance and OAuth token storage are security-sensitive; use platform keyring.
- **Auto-scheduling** that moves meetings without consent is a non-starter; user approval gates.

## Dependencies

- [../feature_matrix.md](../feature_matrix.md) calendar and fitness rows.
- [../MIGRATION_STRATEGY.md](../MIGRATION_STRATEGY.md) for legacy mapping.
- [sidebar.md](./sidebar.md) calendar tile; [control-panel.md](./control-panel.md) for notification channel.

## Open questions

**Resolved (2026-05-28):** see [../ARCHITECTURE_DECISIONS.md](../ARCHITECTURE_DECISIONS.md).

1. **Google Calendar + CalDAV** in v1 backend.
2. Reclaim-style scheduling: **out of scope** (separate app later).
3. Offline cache depth: **TBD** at implementation (default: local SQLite + sync).
