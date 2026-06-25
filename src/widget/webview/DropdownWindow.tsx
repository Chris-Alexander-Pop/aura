// Dropdown WebKit window — top-anchored quick-settings
import { Astal } from "ags/gtk4"
import { registerPanelHoverHandler } from "../../lib/panel-hover"
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
        marginTop: 24,
        width: 1200,
        height: 280,
        visible: false,
        onSetup: (wv) => registerPanelHoverHandler(wv, "dropdownHover", "dropdown"),
    })
}
