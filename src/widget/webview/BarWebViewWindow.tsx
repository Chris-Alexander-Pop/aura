// Always-on vertical strip — React/WebKit replacement for GTK Bar (see docs/roadmap/react-bar-migration.md)
import { Astal, Gdk } from "ags/gtk4"
import { createWebViewWindow } from "./WebViewWindow"

const PAD_V = 12
const PAD_H = 8
/** Narrow column (~56px) + flyout panel (320px) + horizontal padding — keep popouts inside the layer window */
const STRIP_W = 56
const FLYOUT_W = 320
const WINDOW_W = PAD_H + STRIP_W + FLYOUT_W + PAD_H

function monitorHeight(monitor: Gdk.Monitor): number {
    try {
        const r = (monitor as Gdk.Monitor & { get_geometry?: () => { height: number }; geometry?: { height: number } }).get_geometry?.()
            ?? (monitor as Gdk.Monitor & { geometry?: { height: number } }).geometry
        return r?.height ?? 1080
    } catch {
        return 1080
    }
}

export default function BarWebViewWindow(gdkmonitor: Gdk.Monitor) {
    const height = Math.max(480, monitorHeight(gdkmonitor) - PAD_V * 2)
    const safeId = String(gdkmonitor.model ?? "monitor").replace(/[^a-zA-Z0-9_-]/g, "-")
    const name = `bar-wv-${safeId}`

    return createWebViewWindow({
        name,
        page: "#/bar",
        gdkmonitor,
        anchor:
            Astal.WindowAnchor.LEFT |
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.BOTTOM,
        exclusivity: Astal.Exclusivity.IGNORE,
        layer: Astal.Layer.OVERLAY,
        transparentWebView: true,
        marginLeft: PAD_H,
        marginTop: PAD_V,
        marginBottom: PAD_V,
        marginRight: 0,
        width: WINDOW_W,
        height,
        visible: true,
    })
}
