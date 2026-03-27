// Generic GTK4 window wrapper that hosts a WebKitWebView.
// Requires: webkitgtk-6.0 (Arch: sudo pacman -S webkitgtk-6.0)
// @ts-ignore
import WebKit from "gi://WebKit?version=6.0"
import { Astal, Gdk } from "ags/gtk4"
import App from "ags/gtk4/app"

const SIDECAR_URL = "http://localhost:9080"

export interface WebViewWindowOptions {
    name: string
    page: string
    gdkmonitor?: Gdk.Monitor
    anchor: number
    /** Uniform margin (px) — overridden by individual margins */
    margin?: number
    marginLeft?: number
    marginRight?: number
    marginTop?: number
    marginBottom?: number
    width?: number
    height?: number
    visible?: boolean
}

export function createWebViewWindow(opts: WebViewWindowOptions) {
    const {
        name,
        page,
        anchor,
        margin = 0,
        marginLeft,
        marginRight,
        marginTop,
        marginBottom,
        width = 800,
        height = 600,
        visible = false,
    } = opts

    const webview = new WebKit.WebView()
    webview.load_uri(`${SIDECAR_URL}/${page}`)
    webview.set_size_request(width, height)

    const wk_settings = webview.get_settings()
    wk_settings.enable_developer_extras = true
    wk_settings.javascript_can_open_windows_automatically = false

    const win = <window
        name={name}
        visible={visible}
        anchor={anchor}
        margin={margin}
        marginLeft={marginLeft ?? margin}
        marginRight={marginRight ?? margin}
        marginTop={marginTop ?? margin}
        marginBottom={marginBottom ?? margin}
        application={App}
        css="background-color: transparent;"
    >
        {webview}
    </window> as any

    return win
}
