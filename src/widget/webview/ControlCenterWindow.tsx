// ControlCenter WebKit window — right side panel
import { Astal } from "ags/gtk4"
import { createWebViewWindow } from "./WebViewWindow"

export default function ControlCenterWindow() {
    return createWebViewWindow({
        name: "control-center",
        page: "#/control-center",
        anchor:
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.BOTTOM |
            Astal.WindowAnchor.RIGHT,
        margin: 10,
        width: 860,
        height: 720,
    })
}
