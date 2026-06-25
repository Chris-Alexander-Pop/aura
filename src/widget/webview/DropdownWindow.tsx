// Dropdown WebKit window — bottom-right quick-settings card (TOP|RIGHT, above corner)
import { Astal } from "ags/gtk4"
import { registerPanelHoverHandler } from "../../lib/panel-hover"
import { createWebViewWindow } from "./WebViewWindow"

const DROPDOWN_WIDTH = 400
const DROPDOWN_HEIGHT = 280
const INSET = 8

export default function DropdownWindow() {
    return createWebViewWindow({
        name: "dropdown",
        page: "#/dropdown",
        anchor: Astal.WindowAnchor.TOP | Astal.WindowAnchor.RIGHT,
        layer: Astal.Layer.OVERLAY,
        marginTop: 1080 - DROPDOWN_HEIGHT - INSET,
        marginRight: INSET,
        width: DROPDOWN_WIDTH,
        height: DROPDOWN_HEIGHT,
        visible: false,
        onSetup: (wv) => registerPanelHoverHandler(wv, "dropdownHover", "dropdown"),
    })
}
