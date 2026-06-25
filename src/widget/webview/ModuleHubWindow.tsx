import { Astal } from "ags/gtk4"
import { registerPanelHoverHandler } from "../../lib/panel-hover"
import { createWebViewWindow } from "./WebViewWindow"

const MODULE_HUB_HEIGHT = 400

export default function ModuleHubWindow() {
    return createWebViewWindow({
        name: "module-hub",
        page: "#/module-hub",
        anchor: Astal.WindowAnchor.TOP,
        marginTop: 10,
        marginLeft: 640,
        width: 640,
        height: MODULE_HUB_HEIGHT,
        visible: false,
        onSetup: (wv) => registerPanelHoverHandler(wv, "moduleHubHover", "module-hub"),
    })
}
