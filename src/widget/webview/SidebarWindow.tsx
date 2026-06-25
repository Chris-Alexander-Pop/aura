import { Astal } from "ags/gtk4"
import { registerPanelHoverHandler } from "../../lib/panel-hover"
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
        marginLeft: 80,
        width: 360,
        height: 720,
        visible: false,
        onSetup: (wv) => registerPanelHoverHandler(wv, "sidebarHover", "sidebar"),
    })
}
