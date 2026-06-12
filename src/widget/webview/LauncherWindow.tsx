import { createWebViewWindow } from "./WebViewWindow"

export default function LauncherWindow() {
    return createWebViewWindow({
        name: "launcher",
        page: "#/launcher",
        hyprlandToplevel: true,
        title: "Aura Launcher",
        width: 640,
        height: 420,
        visible: false,
    })
}
