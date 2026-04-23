# Lock panel (roadmap)

## Summary

The **lock and session** experience: true **system sleep** (S3/idle) where hardware allows, **biometric** unlock (fingerprint via **fprintd** or equivalent), a **hardened** and **clutter-free** lock UI, and **user-configurable** visibility of **notifications, calendar snippets, and media** on the lock screen.

Aligns with **session** entries in [../feature_matrix.md](../feature_matrix.md) and power in [`sidecar/src/services/power.rs`](../../sidecar/src/services/power.rs).

## Current baseline

`hyprlock`, `swaylock`, or **Kitty+script**-class lockers may be external today; Aura’s role is **integration, defaults, and UI spec**. Caelestia’s **session** module is the UX reference. [lock-panel] is partly operational (OS level), partly shell.

## Goals

### Proper sleep mode

**Suspend** and **hibernate** (if swap configured) with **lid**, **timeout**, and **inhibit** (presentations) hooks. Expose **why** the system is awake (inhibit locks) in [system-debugging-popup.md](./system-debugging-popup.md).

### Fingerprint working / etc..

**PAM** stack integration: fingerprint **before** and **with** password fallback; **clear** errors when the reader is busy or not enrolled. Ties to [control-panel.md](./control-panel.md) security/fingerprint.

### Clean up and more secure, configurable what should be visible

- **No sensitive** previews by default (message bodies, **OTP**).
- **Configurable**: show **clock only**, or **next event title** from [calendar-panel.md](./calendar-panel.md) if user opts in.
- **Reduce** information leaks from **on-screen keyboards** and **URL** previews in notifications.

## Out of scope / risks

- **Biometric** bypass bugs are high severity; follow **PAM** best practices and **session** security guides.
- **Hibernate** on **encrypted** swap requires **correct** setup; document **prereqs** rather than faking it.

## Dependencies

- [control-panel.md](./control-panel.md) (security, session).
- [system-and-input-foundation.md](./system-and-input-foundation.md) (login chain **before** lock, too).
- Compositor **lock** protocol in Hyprland + `hyprlock`/`swaylock` choice.

## Open questions

1. **Single** lock implementation (`hyprlock` theme **owned** by Aura) vs **pluggable**?
2. **Music** controls on lock: allowed or always disabled?
3. **Smart unlock** (Bluetooth, phone proximity): desired or out?
