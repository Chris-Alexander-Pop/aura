// ControlCenter — normal Gtk toplevel so Hyprland can tile / float / close it.
import { registerPanelHoverHandler } from "../../lib/panel-hover"
import { createWebViewWindow } from "./WebViewWindow"

export default function ControlCenterWindow() {
    return createWebViewWindow({
        name: "control-center",
        page: "#/control-center",
        hyprlandToplevel: true,
        title: "Aura Control Center",
        width: 860,
        height: 720,
        visible: false,
        onSetup: (wv) => registerPanelHoverHandler(wv, "controlCenterHover", "control-center"),
    })
}
