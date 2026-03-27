// Dropdown WebKit window — top-anchored quick-settings
import { Astal } from "ags/gtk4"
import { createWebViewWindow } from "./WebViewWindow"

export default function DropdownWindow() {
    return createWebViewWindow({
        name: "dropdown",
        page: "#/dropdown",
        anchor:
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.LEFT |
            Astal.WindowAnchor.RIGHT,
        margin: 8,
        width: 1200,
        height: 220,
    })
}
