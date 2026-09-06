// Calendar WebKit window — full-height right panel (480px, month-first UI)
import Gdk from "gi://Gdk?version=4.0"
import { Astal } from "ags/gtk4"
import { registerPanelHoverHandler } from "../../lib/panel-hover"
import { createWebViewWindow } from "./WebViewWindow"

const CALENDAR_WIDTH = 480

/** Catppuccin Mocha mantle — matches WebKit bg so rounded corners need no transparency. */
function setPanelWebViewBg(webview: unknown) {
    try {
        const rgba = new Gdk.RGBA()
        rgba.red = 30 / 255
        rgba.green = 30 / 255
        rgba.blue = 46 / 255
        rgba.alpha = 1
        ;(webview as { set_background_color?: (c: Gdk.RGBA) => void }).set_background_color?.(rgba)
    } catch {
        /* older WebKit */
    }
}

export default function CalendarWindow() {
    return createWebViewWindow({
        name: "calendar",
        page: "#/calendar",
        anchor:
            Astal.WindowAnchor.RIGHT |
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.BOTTOM,
        margin: 8,
        marginRight: 8,
        width: CALENDAR_WIDTH,
        height: 900,
        visible: false,
        onSetup: (wv) => {
            setPanelWebViewBg(wv)
            registerPanelHoverHandler(wv, "calendarHover", "calendar")
        },
    })
}
