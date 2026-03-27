// Sidebar WebKit window — always visible, sits to the right of the 52px bar
import { Astal } from "ags/gtk4"
import { createWebViewWindow } from "./WebViewWindow"

const BAR_WIDTH = 58 // bar min-width (50px) + padding (4+4px)

export default function SidebarWindow() {
    return createWebViewWindow({
        name: "sidebar",
        page: "#/sidebar",
        anchor:
            Astal.WindowAnchor.LEFT |
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.BOTTOM,
        marginLeft: BAR_WIDTH + 8, // push it right of the bar
        marginTop: 8,
        marginBottom: 8,
        marginRight: 0,
        width: 340,
        height: 720,
        visible: true, // always on
    })
}
