# Exocortex panel (roadmap)

## Summary

**Placeholder (stub).** Reserved name for an **externalized second brain**: capture **ideas**, **read-later**, **references**, and **links** into a **queryable** system that complements [calendar-panel.md](./calendar-panel.md) and **todo** flows without being just another task list. The todo line was `..`; product shape is **TBD**.

“Exocortex” here means **trusted personal knowledge**, not raw social feeds.

## Current baseline

No dedicated feature row in [../feature_matrix.md](../feature_matrix.md). Possible overlap with future **notes** or **logs** UX.

## Goals

### Stub / TBD

**Candidate scope**: 

- **Capture** via global shortcut → **inbox**, then triage to **projects** or **literature**.
- **Search** across title, tags, and **full text** if local files.
- **Sync** optional (encrypted) to a **self-hosted** target; no mandatory cloud.

**Explicitly undefined**: schema (Markdown files vs DB), AI **embeddings** vs pure **keyword**, and collaboration.

## Out of scope / risks

- **Embedding** models and **LLM** features require clear **privacy** toggles and **offline** modes.
- Avoid becoming a **duplicate** of Notion/Obsidian unless **differentiation** is clear (e.g. **shell-native** capture).

## Dependencies

- [vault-panel.md](./vault-panel.md) for **where files live** and **encryption**.
- [communication-panel.md](./communication-panel.md) if capturing from **messages**.
- [top-dropdown.md](./top-dropdown.md) for **quick capture** entry.

## Open questions

1. **File-over-app** (Markdown/git) as the **source of truth**?
2. **Graph view** (nodes/edges) in v1 or later?
3. Relationship to [marginal-gains-panel.md](./marginal-gains-panel.md) **study** notes—same DB or separate?
