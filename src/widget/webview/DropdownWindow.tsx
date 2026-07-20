// Dropdown WebKit window — bottom-right quick-settings card (TOP|RIGHT, above corner)
import Gdk from "gi://Gdk?version=4.0"
import { Astal } from "ags/gtk4"
import { registerPanelHoverHandler } from "../../lib/panel-hover"
import { createWebViewWindow } from "./WebViewWindow"

const DROPDOWN_WIDTH = 400
const DROPDOWN_HEIGHT = 280
const INSET = 8

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
        onSetup: (wv) => {
            setPanelWebViewBg(wv)
            registerPanelHoverHandler(wv, "dropdownHover", "dropdown")
        },
    })
}
