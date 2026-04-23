// Sidebar WebKit window — layer shell strip beside the bar (hidden until toggled)
import { Astal } from "ags/gtk4"
import { createWebViewWindow } from "./WebViewWindow"

/** Align with BarWebViewWindow: marginLeft (8) + strip column (56) + gap before sidebar */
const BAR_PAD_H = 8
const BAR_STRIP_W = 56
const SIDEBAR_GAP = 8
const SIDEBAR_MARGIN_LEFT = BAR_PAD_H + BAR_STRIP_W + SIDEBAR_GAP

export default function SidebarWindow() {
    return createWebViewWindow({
        name: "sidebar",
        page: "#/sidebar",
        anchor:
            Astal.WindowAnchor.LEFT |
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.BOTTOM,
        marginLeft: SIDEBAR_MARGIN_LEFT,
        marginTop: 8,
        marginBottom: 8,
        marginRight: 0,
        width: 340,
        height: 720,
        visible: false,
    })
}
