// Sidebar WebKit window — left overlay (next to bar)
import { Astal } from "ags/gtk4"
import { createWebViewWindow } from "./WebViewWindow"

export default function SidebarWindow() {
    return createWebViewWindow({
        name: "sidebar",
        page: "#/sidebar",
        anchor:
            Astal.WindowAnchor.LEFT |
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.BOTTOM,
        margin: 8,
        width: 340,
        height: 720,
    })
}
