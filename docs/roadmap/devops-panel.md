# Devops panel (roadmap)

## Summary

A **DevOps** surface in Aura to **view and manage** deployments and infrastructure across **local** (Docker/Podman, Compose), **home lab** (single nodes, lightweight Kubernetes, hypervisor links), and **cloud** (provider APIs or generic health checks). Emphasis on **resource consumption** (CPU, memory, restarts, cost where available), **health**, and **actions** (restart, scale, open logs) where safe—surfaced in `ui/` and backed by [`sidecar/src/services/devops.rs`](../../sidecar/src/services/devops.rs) per [../feature_matrix.md](../feature_matrix.md).

## Current baseline

[../feature_matrix.md](../feature_matrix.md) lists **DevOps** with a sidecar service and client `getDevopsStatus` in [`src/lib/sidecar.ts`](../../src/lib/sidecar.ts); **UI** is not implemented. [../COMPONENT_MAPPING.md](../COMPONENT_MAPPING.md) may map legacy Caelestia `devops/`.

## Goals

### View + manage local, home, and cloud deployments / resource consumption / etc.

- **Inventory**: environments (dev/stage/prod), clusters, namespaces, or compose stacks—user-defined **connections** with secrets in the OS keyring.
- **Metrics**: CPU, memory, restart counts, error rates, and **cost** if billing or tag data is available.
- **Actions**: read-only by default; **write** actions (deploy, restart) behind confirmation and least-privilege credentials.

**Candidate scope**: connectors for **local** containers, **Kubernetes** via `kubectl` with a configured kubeconfig, and **one** major cloud API—plus a generic **HTTP health** row for anything else.

## Out of scope / risks

- **Cloud root keys** must not live in plain text; use the keyring and short-lived tokens where possible.
- **kubectl** and **docker** from the UI can be destructive; default to read-only views and document required roles.

## Dependencies

- Sidecar [`devops.rs`](../../sidecar/src/services/devops.rs) and TS client methods.
- [top-dropdown.md](./top-dropdown.md) for quick status.
- Host tools (`docker`, `kubectl`, etc.) with clear UI when binaries are missing.

## Open questions

1. Is **Docker socket** access on the workstation acceptable, or should the panel use a **remote** API only?
2. **Multi-cloud / multi-cluster**: single combined view or separate profiles?
3. **Cost** tracking: first-class in v1 or deferred?
