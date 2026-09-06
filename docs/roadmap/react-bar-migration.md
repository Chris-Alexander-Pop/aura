# Always-on vertical bar — GTK → React/WebKit

## Summary

The persistent strip was implemented in **GTK** ([`src/widget/bar/Bar.tsx`](../../src/widget/bar/Bar.tsx)). This migration moves **pixel rendering** to **React** inside a **WebKit** layer-shell window ([`BarWebViewWindow.tsx`](../../src/widget/webview/BarWebViewWindow.tsx)), keeping a **thin GTK host** only for Wayland anchors and exclusivity.

## Tradeoffs

| Topic | GTK bar | React bar |
|--------|---------|-----------|
| Layer-shell | Native | Same — WebKit lives inside `Astal.Window` |
| Tray (StatusNotifier) | `AstalTray` GObject | Not available in DOM — needs **sidecar D-Bus** later or **GTK hybrid** |
| Hyprland | `hyprland.ts` (GJS) | **`Hyprland.*` JSON-RPC** in sidecar (`hyprctl`) |
| Session / spawn | Direct `GLib.spawn` | **Allowlisted** `Session.*`, `Aura.ToggleWindow`, `Apps.Launch` RPC |
| Footprint | One process | **One WebKit per monitor** |

## Rollback

Set environment variable **`AURA_GTK_BAR=1`** before launching AGS to restore the legacy GTK bar instead of WebKit strips ([`app.ts`](../../app.ts)).

## Blockers addressed

- **Tray**: UI shows a placeholder until StatusNotifier is exposed via sidecar.
- **Hyprland**: Provided by [`Hyprland.*`](../../sidecar/src/services/hyprland.rs) methods consumed from [`api.ts`](../../ui/src/lib/api.ts).
