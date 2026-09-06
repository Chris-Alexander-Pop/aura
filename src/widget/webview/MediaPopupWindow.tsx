import { Astal } from "ags/gtk4"
import { registerPanelHoverHandler } from "../../lib/panel-hover"
import { createWebViewWindow } from "./WebViewWindow"

const MEDIA_WIDTH = 88
const MEDIA_HEIGHT = 220

export default function MediaPopupWindow() {
    return createWebViewWindow({
        name: "media-popup",
        page: "#/media-popup",
        anchor: Astal.WindowAnchor.TOP | Astal.WindowAnchor.RIGHT,
        layer: Astal.Layer.OVERLAY,
        transparentWebView: true,
        marginRight: 0,
        marginTop: 456,
        width: MEDIA_WIDTH,
        height: MEDIA_HEIGHT,
        visible: false,
        onSetup: (wv) => {
            registerPanelHoverHandler(wv, "mediaPopupHover", "media-popup")
        },
    })
}
