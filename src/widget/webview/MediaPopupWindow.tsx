import { Astal } from "ags/gtk4"
import { registerPanelHoverHandler } from "../../lib/panel-hover"
import { createWebViewWindow } from "./WebViewWindow"

export default function MediaPopupWindow() {
    return createWebViewWindow({
        name: "media-popup",
        page: "#/media-popup",
        anchor:
            Astal.WindowAnchor.RIGHT |
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.BOTTOM,
        marginRight: 8,
        marginTop: 350,
        marginBottom: 350,
        width: 320,
        height: 380,
        visible: false,
        transparentWebView: true,
        onSetup: (wv) => registerPanelHoverHandler(wv, "mediaPopupHover", "media-popup"),
    })
}
