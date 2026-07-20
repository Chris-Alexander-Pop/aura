import Gdk from "gi://Gdk?version=4.0"
import { Astal } from "ags/gtk4"
import { registerPanelHoverHandler } from "../../lib/panel-hover"
import { createWebViewWindow } from "./WebViewWindow"

const MODULE_HUB_HEIGHT = 540

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

export default function ModuleHubWindow() {
    return createWebViewWindow({
        name: "module-hub",
        page: "#/module-hub",
        anchor: Astal.WindowAnchor.TOP,
        layer: Astal.Layer.OVERLAY,
        marginTop: 10,
        marginLeft: 720,
        width: 720,
        height: MODULE_HUB_HEIGHT,
        visible: false,
        onSetup: (wv) => {
            setPanelWebViewBg(wv)
            registerPanelHoverHandler(wv, "moduleHubHover", "module-hub")
        },
    })
}
