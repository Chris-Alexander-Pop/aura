// Always-on left bar strip (#/bar) + separate flyout overlay window.
//
// Splitting into two windows eliminates the "wide transparent WebKit layer
// steals pointer input" problem: the strip window is narrow and EXCLUSIVE
// (reserves space, receives input), while the flyout window is a separate
// OVERLAY/IGNORE window that only appears when needed.
//
// Communication: strip React → AGS via WebKit message handler "barFlyout"
//                flyout React → AGS via WebKit message handler "barFlyoutHover"
// @ts-ignore
import WebKit from "gi://WebKit?version=6.0"
import { Astal, Gdk } from "ags/gtk4"
import App from "ags/gtk4/app"
import { createWebViewWindow } from "./WebViewWindow"
import hyprland from "../../lib/hyprland"

import { BAR_STRIP_WIDTH_PX, monitorTag } from "../../lib/monitor"

const STRIP_W = BAR_STRIP_WIDTH_PX
const FLYOUT_W = 320 // max popout width; individual panels use narrower content inside
const CLOSE_DELAY_MS = 250

type AnyWv = WebKit.WebView & {
    load_uri: (uri: string) => void
    set_size_request: (w: number, h: number) => void
    get_settings: () => Record<string, unknown>
    set_background_color?: (c: Gdk.RGBA) => void
    evaluate_javascript?: (s: string, len: number, w: null, src: null, cancel: null, cb: null) => void
    get_user_content_manager: () => {
        register_script_message_handler: (name: string, world: null) => boolean
        connect: (sig: string, cb: (mgr: unknown, val: unknown) => void) => number
    }
    connect: (sig: string, cb: (...args: unknown[]) => unknown) => number
}

function monitorHeight(monitor: Gdk.Monitor): number {
    try {
        const r = (monitor as Gdk.Monitor & {
            get_geometry?: () => { height: number }
            geometry?: { height: number }
        }).get_geometry?.() ?? (monitor as Gdk.Monitor & { geometry?: { height: number } }).geometry
        return r?.height ?? 1080
    } catch {
        return 1080
    }
}

export default function BarWebViewWindow(gdkmonitor: Gdk.Monitor) {
    const height = Math.max(480, monitorHeight(gdkmonitor))
    const safeId = monitorTag(gdkmonitor)
    const flyoutName = `bar-flyout-${safeId}`

    // ── Close timer (shared between strip-leave and flyout-leave) ─────────────
    let closeTimer: ReturnType<typeof setTimeout> | null = null
    let flyoutReady = false
    let pendingPayload: { panel: string; y: number } | null = null

    const cancelClose = () => {
        if (closeTimer != null) { clearTimeout(closeTimer); closeTimer = null }
    }

    const scheduleClose = () => {
        cancelClose()
        closeTimer = setTimeout(() => {
            const win = App.get_window(flyoutName) as { visible: boolean } | null
            if (win) win.visible = false
        }, CLOSE_DELAY_MS)
    }

    // ── Flyout WebView (built manually to keep a reference) ──────────────────
    const flyoutWv = new WebKit.WebView() as AnyWv
    flyoutWv.connect("context-menu", (_wv, menu) => {
        try {
            const ctx = menu as {
                n_items?: () => number
                item_at_index?: (i: number) => unknown
                remove?: (item: unknown) => void
            }
            const n = ctx.n_items?.() ?? 0
            for (let i = n - 1; i >= 0; i--) {
                const item = ctx.item_at_index?.(i)
                if (item) ctx.remove?.(item)
            }
        } catch { /* ignore */ }
        return true
    })
    flyoutWv.load_uri(`http://localhost:9080/?v=${Date.now()}#/bar-flyout`)
    flyoutWv.set_size_request(FLYOUT_W, height)
    const flyoutSettings = flyoutWv.get_settings()
    flyoutSettings["enable_developer_extras"] = true
    flyoutSettings["javascript_can_open_windows_automatically"] = false
    try {
        const rgba = new Gdk.RGBA()
        rgba.red = 0; rgba.green = 0; rgba.blue = 0; rgba.alpha = 0
        flyoutWv.set_background_color?.(rgba)
    } catch { /* older WebKit */ }

    const dispatchToFlyout = (panel: string, y: number) => {
        const script = `window.dispatchEvent(new CustomEvent('aura-flyout',{detail:${JSON.stringify({ panel, y })}}))`
        flyoutWv.evaluate_javascript?.(script, -1, null, null, null, null)
    }

    const showFlyout = (panel: string, y: number) => {
        cancelClose()
        const win = App.get_window(flyoutName) as { visible: boolean } | null
        if (!win) return
        if (flyoutReady) {
            dispatchToFlyout(panel, y)
            win.visible = true
        } else {
            pendingPayload = { panel, y }
        }
    }

    // Show once the SPA has bootstrapped
    flyoutWv.connect('load-changed', (_wv: unknown, event: unknown) => {
        if (event === 3 /* WebKit.LoadEvent.FINISHED */) {
            flyoutReady = true
            if (pendingPayload) {
                const p = pendingPayload
                pendingPayload = null
                dispatchToFlyout(p.panel, p.y)
                const win = App.get_window(flyoutName) as { visible: boolean } | null
                if (win) win.visible = true
            }
        }
    })

    // Flyout hover messages: cancel/restart the close timer from within the flyout
    try {
        const flyMgr = flyoutWv.get_user_content_manager()
        flyMgr.register_script_message_handler('barFlyoutHover', null)
        flyMgr.connect('script-message-received::barFlyoutHover', (_m: unknown, jsVal: unknown) => {
            try {
                const msg = (jsVal as { to_string?: () => string }).to_string?.() ?? String(jsVal)
                if (msg === 'enter') cancelClose()
                else scheduleClose()
            } catch { /* ignore */ }
        })
    } catch (e) { print(`barFlyoutHover setup error: ${e}`) }

    // ── Flyout window (below strip in z-order; emerges from behind the bar) ──
    const flyoutWin = <window
        name={flyoutName}
        gdkmonitor={gdkmonitor}
        visible={false}
        anchor={
            Astal.WindowAnchor.LEFT |
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.BOTTOM
        }
        exclusivity={Astal.Exclusivity.IGNORE}
        layer={Astal.Layer.TOP}
        marginLeft={STRIP_W}
        application={App}
        css="background-color: transparent;"
    >
        {flyoutWv}
    </window> as unknown

    void flyoutWin

    // ── Strip window — EXCLUSIVE + OVERLAY so flyouts stay underneath ───────
    return createWebViewWindow({
        name: `bar-wv-${safeId}`,
        page: "#/bar",
        gdkmonitor,
        anchor:
            Astal.WindowAnchor.LEFT |
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.BOTTOM,
        exclusivity: Astal.Exclusivity.EXCLUSIVE,
        layer: Astal.Layer.OVERLAY,
        transparentWebView: true,
        margin: 0,
        width: STRIP_W,
        height,
        visible: true,
        onSetup: (wv) => {
            try {
                const webview = wv as AnyWv

                const pushWorkspaceToBar = (id: number) => {
                    if (!Number.isFinite(id) || id <= 0) return
                    const script =
                        `window.dispatchEvent(new CustomEvent('aura-hypr-workspace',{detail:{id:${id}}}))`
                    webview.evaluate_javascript?.(script, -1, null, null, null, null)
                }
                hyprland.connect("focused-workspace-changed", (_svc: unknown, id: number) => {
                    pushWorkspaceToBar(id)
                })

                const mgr = webview.get_user_content_manager()
                mgr.register_script_message_handler('barFlyout', null)
                mgr.connect('script-message-received::barFlyout', (_m: unknown, jsVal: unknown) => {
                    try {
                        const jv = jsVal as {
                            to_json?: (indent: number) => string
                            to_string?: () => string
                        }
                        // to_json(0) serialises JS objects to JSON correctly.
                        // If the page posted a pre-stringified string instead of a
                        // plain object, to_json wraps it in extra quotes — in that
                        // case to_string() gives us the raw string we want.
                        const raw = jv.to_json?.(0) ?? jv.to_string?.() ?? ''
                        // raw either looks like {"open":true,...} (object) or
                        // like "\"{ escaped json }\"" (double-encoded string).
                        const str = raw.startsWith('"') ? JSON.parse(raw) as string : raw
                        const data = JSON.parse(str) as { open: boolean; panel?: string; y?: number }
                        if (data.open && data.panel && data.y != null) {
                            showFlyout(data.panel, data.y)
                        } else {
                            scheduleClose()
                        }
                    } catch (e) { print(`barFlyout message error: ${e}`) }
                })
            } catch (e) { print(`barFlyout setup error: ${e}`) }
        },
    })
}
