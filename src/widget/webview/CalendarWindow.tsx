// Calendar WebKit window — full-height left panel
import { Astal } from "ags/gtk4"
import { createWebViewWindow } from "./WebViewWindow"

export default function CalendarWindow() {
    return createWebViewWindow({
        name: "calendar",
        page: "#/calendar",
        anchor:
            Astal.WindowAnchor.LEFT |
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.BOTTOM,
        margin: 8,
        width: 420,
        height: 720,
    })
}
