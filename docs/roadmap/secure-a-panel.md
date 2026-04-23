# Secure A panel (roadmap)

## Summary

A **research and analysis** environment for **psychological profiles**, **threat/“vector”** models, and **structured research**—framed for **defensive** and **academic** use: OSINT, **security** education, and **operational** awareness, **not** harassment. This is **sensitive**; the UI must **discourage** misuse with **ethics** copy, **access** controls, and **audit** expectations.

**Note**: “Secure A” is a **working name** from the personal todo; rename if it ships publicly.

## Current baseline

[../feature_matrix.md](../feature_matrix.md) lists a **Security** service ([`sidecar/src/services/security.rs`](../../sidecar/src/services/security.rs)) with UI not implemented. [../COMPONENT_MAPPING.md](../COMPONENT_MAPPING.md) may map legacy security modules. This panel may **extend** that service for **user research vaults** or stay a **separate** app with Aura as **launcher + notes shell**.

## Goals

### psych profiles / vectors / research..

**Interpretation (high level)**:

- **Profiles**: **structured** fields (traits, stressors, communication style) for **fictional** or **public** figures in **authorized** contexts (writing, red-team **scenarios**), with **no** “creep” feature set.
- **Vectors**: **attack** or **influence** **vectors** in **defensive** terms (e.g. phishing, social engineering **mitigations**), not a **targeting** tool.
- **Research**: **bibliography**, **claims → sources**, and **uncertainty** tags.

**Candidate scope**: **markdown** or **form**-based **notebooks** with **tags** and **export**; **optional** **graph** of **concepts** (not people-labeled).

## Out of scope / risks

- **Doxxing**, **stalking**, and **non-consensual** profiling: **forbidden** by product policy; **report** and **refuse** patterns that look like that.
- **Data** in this panel may be **legally** sensitive; **encrypt at rest** and **lock** with [lock-panel.md](./lock-panel.md).

## Dependencies

- [vault-panel.md](./vault-panel.md) for **storage** and **encryption**.
- [pentest-panel.md](./pentest-panel.md) for **lab**-style **defensive** exercises (keep boundaries clear).
- [exocortex-panel.md](./exocortex-panel.md) for **long-form** **synthesis** if that product exists.

## Open questions

1. Is this panel **intentionally** **internal-only** and **never** shipped to a **public** repo?
2. **IRL** “profiling” of **colleagues**: **hard** **no** in UI—**confirm** as policy.
3. **Export** formats for **academic** **citation** (BibTeX) in v1?
