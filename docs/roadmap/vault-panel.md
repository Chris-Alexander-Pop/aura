# Vault panel (roadmap)

## Summary

A **unified “vault”** for **cloud storage and files**: aggregate multiple providers (S3, Google Drive, Nextcloud, …) in one **browsable** surface, use it as a **first-class “external” layer** in the file workflow, and support **fast transfer** between machines and **media servers** (Plex/Jellyfin-adjacent paths), **automatic system backups**, and **security/encryption/sharing** controls.

This is likely a **large** `ui/` panel with heavy **sidecar** and optional **FUSE**/`rclone` integration; performance and **secrets** handling are the hard parts.

## Current baseline

Not a direct row in [../feature_matrix.md](../feature_matrix.md); [../COMPONENT_MAPPING.md](../COMPONENT_MAPPING.md) may not cover it. Some **automation** and **security** services exist in sidecar for related hooks.

## Goals

### App to combine all the cloud storages and such, use it for external storage

**Provider registry**: add accounts, optional **rclone** config import, **unified** directory tree (virtual root per provider) or **favorites** list. **Streaming** of large files vs local cache policy.

### Quicktransfer across media servers and whatever

**Send to** / **pull from** other hosts: **rsync**-class jobs with progress UI, **tailscale**-aware host list (candidate), and shortcuts for “push current folder to media server path.”

### Automatic backups from system

**Policy-driven** backups: what to include (`/home` subsets, config dirs), **schedule**, **retention**, and **verification** (test restore of a file). **Integration** with the same storage targets as above.

### Security / encryption / sharing..

**At-rest** encryption (client-side keys), **signed links** for share, and **expiring** URLs. **Audit** log of shares.

## Out of scope / risks

- **Sync engines** are easy to get wrong (conflicts, data loss); need **versioning** and “undo.”
- **API keys** must never appear in shell logs; **OAuth** in browser where possible.
- **Full** system imaging may belong in dedicated tools; scope **user data** first.

## Dependencies

- OS keyring, optional `rclone`, `restic`/`borg` (TBD) as backends the panel **orchestrates**.
- [lock-panel.md](./lock-panel.md) and [system-debugging-popup.md](./system-debugging-popup.md) for **restore** entry points.
- [control-panel.md](./control-panel.md) **security** area for **AV/scan** after bulk restore.

## Open questions

1. **Virtual mount** in the file manager vs **browser-only** in Aura?
2. **Backup** engine: wrap **existing** CLI or ship **minimal** custom?
3. **Multi-user** machine: separate vault profiles?
