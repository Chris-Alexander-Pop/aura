// Generic GTK4 window wrapper that hosts a WebKitWebView.
// Requires: webkitgtk-6.0 (Arch: sudo pacman -S webkitgtk-6.0)
// @ts-ignore
import WebKit from "gi://WebKit?version=6.0"
import Gtk from "gi://Gtk?version=4.0"
import { Gdk } from "ags/gtk4"
import App from "ags/gtk4/app"

const SIDECAR_URL = "http://localhost:9080"

export interface WebViewWindowOptions {
    name: string
    page: string
    /**
     * Plain XDG toplevel (Gtk.ApplicationWindow) — Hyprland can tile, float, and
     * close it like any app. When false/omitted, uses Astal.Window (layer shell).
     */
    hyprlandToplevel?: boolean
    /** Title bar / compositor list text when hyprlandToplevel is set */
    title?: string
    gdkmonitor?: Gdk.Monitor
    /** Layer-shell edges (required unless hyprlandToplevel). */
    anchor?: number
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

function makeWebView(page: string, width: number, height: number, fixedMinSize: boolean) {
    const webview = new WebKit.WebView()
    webview.load_uri(`${SIDECAR_URL}/${page}`)
    if (fixedMinSize) {
        webview.set_size_request(width, height)
    } else {
        webview.set_hexpand(true)
        webview.set_vexpand(true)
    }
    const wk_settings = webview.get_settings()
    wk_settings.enable_developer_extras = true
    wk_settings.javascript_can_open_windows_automatically = false
    return webview
}

function applyTransparentWindowCss(widget: Gtk.Widget) {
    const provider = new Gtk.CssProvider()
    provider.load_from_string("* { background-color: transparent; }")
    widget.get_style_context().add_provider(provider, Gtk.STYLE_PROVIDER_PRIORITY_USER)
}

function createHyprlandWebViewWindow(opts: WebViewWindowOptions) {
    const {
        name,
        page,
        title = "Aura",
        width = 800,
        height = 600,
        visible = false,
    } = opts

    const webview = makeWebView(page, width, height, false)
    const win = new Gtk.ApplicationWindow({ application: App })
    win.name = name
    win.title = title
    win.set_default_size(width, height)
    win.set_child(webview)
    win.visible = visible
    applyTransparentWindowCss(win)

    win.connect("close-request", () => {
        win.visible = false
        return true
    })

    return win
}

export function createWebViewWindow(opts: WebViewWindowOptions) {
    if (opts.hyprlandToplevel) {
        return createHyprlandWebViewWindow(opts)
    }

    const anchor = opts.anchor
    if (anchor === undefined) {
        throw new Error(`WebViewWindow "${opts.name}": anchor is required unless hyprlandToplevel is true`)
    }

    const {
        name,
        page,
        margin = 0,
        marginLeft,
        marginRight,
        marginTop,
        marginBottom,
        width = 800,
        height = 600,
        visible = false,
    } = opts

    const webview = makeWebView(page, width, height, true)

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
