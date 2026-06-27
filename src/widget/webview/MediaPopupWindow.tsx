import Gdk from "gi://Gdk?version=4.0"
import { Astal } from "ags/gtk4"
import { registerPanelHoverHandler } from "../../lib/panel-hover"
import { createWebViewWindow } from "./WebViewWindow"

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

export default function MediaPopupWindow() {
    return createWebViewWindow({
        name: "media-popup",
        page: "#/media-popup",
        anchor:
            Astal.WindowAnchor.RIGHT |
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.BOTTOM,
        layer: Astal.Layer.TOP,
        marginRight: 8,
        marginTop: 350,
        marginBottom: 350,
        width: 320,
        height: 380,
        visible: false,
        onSetup: (wv) => {
            setPanelWebViewBg(wv)
            registerPanelHoverHandler(wv, "mediaPopupHover", "media-popup")
        },
    })
}
