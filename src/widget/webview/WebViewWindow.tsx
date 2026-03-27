// Generic GTK4 window wrapper that hosts a WebKitWebView.
// Each panel page in the React UI is served from the sidecar at localhost:9080.
// @ts-ignore
import WebKit from "gi://WebKit"
import { Astal, Gdk } from "ags/gtk4"
import App from "ags/gtk4/app"

const SIDECAR_URL = "http://localhost:9080"

export interface WebViewWindowOptions {
    name: string
    page: string             // e.g. "#/control-center"
    gdkmonitor?: Gdk.Monitor
    anchor: number
    margin?: number
    width?: number
    height?: number
    exclusive?: boolean
}

export function createWebViewWindow(opts: WebViewWindowOptions) {
    const {
        name,
        page,
        anchor,
        margin = 0,
        width = 800,
        height = 600,
    } = opts

    const webview = new WebKit.WebView()
    webview.load_uri(`${SIDECAR_URL}/${page}`)
    webview.set_size_request(width, height)

    // Trust localhost completely
    const wk_settings = webview.get_settings()
    wk_settings.enable_developer_extras = true
    wk_settings.javascript_can_open_windows_automatically = false

    const win = <window
        name={name}
        visible={false}
        anchor={anchor}
        margin={margin}
        application={App}
        css="background-color: transparent;"
    >
        {webview}
    </window> as any

    return win
}
