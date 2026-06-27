import Astal from "gi://Astal?version=4.0"
import Gdk from "gi://Gdk?version=4.0"
import Gtk from "gi://Gtk?version=4.0"
import App from "ags/gtk4/app"
import {
    moduleHubLayout,
    moduleHubLeftEdgeLayout,
    verticalCenterMargins,
    bottomRightCardLayout,
} from "./monitor"

const DEFAULT_CLOSE_MS = 450

const SIDEBAR_HEIGHT = 400
const DROPDOWN_WIDTH = 400
const DROPDOWN_HEIGHT = 280
const MEDIA_WIDTH = 320
const MEDIA_HEIGHT = 380

const HOVER_PANEL_NAMES = new Set(["module-hub", "dropdown", "media-popup"])

export type ModuleHubTriggerMode = "left_edge" | "top_third" | "none"

let moduleHubTriggerMode: ModuleHubTriggerMode = "top_third"
let hideShellOnFullscreen = true

export function setModuleHubTriggerMode(mode: ModuleHubTriggerMode): void {
    moduleHubTriggerMode = mode
}

export function setHideShellOnFullscreen(hide: boolean): void {
    hideShellOnFullscreen = hide
}

export function getHideShellOnFullscreen(): boolean {
    return hideShellOnFullscreen
}

type AuraWindow = Gtk.Window & {
    visible?: boolean
    show?: () => void
    hide?: () => void
    set_gdkmonitor?: (m: Gdk.Monitor) => void
    set_anchor?: (a: number) => void
    set_margin_left?: (n: number) => void
    set_margin_right?: (n: number) => void
    set_margin_top?: (n: number) => void
    set_margin_bottom?: (n: number) => void
    get_child?: () => Gtk.Widget | null
}

const timers = new Map<string, ReturnType<typeof setTimeout>>()

function getWindow(name: string): AuraWindow | null {
    return App.get_window(name) as AuraWindow | null
}

function resizeWebViewChild(win: AuraWindow, width: number, height: number) {
    const child = win.get_child?.()
    if (child && "set_size_request" in child) {
        ;(child as Gtk.Widget).set_size_request(width, height)
    }
}

/** Position hover panels on the monitor whose trigger was entered. */
function applyPanelLayout(
    name: string,
    win: AuraWindow,
    monitor: Gdk.Monitor,
    opts?: { moduleHubTop?: boolean }
): void {
    win.set_gdkmonitor?.(monitor)

    switch (name) {
        case "module-hub": {
            const useTop =
                opts?.moduleHubTop === true ||
                moduleHubTriggerMode === "top_third" ||
                moduleHubTriggerMode === "none"
            if (!useTop && moduleHubTriggerMode === "left_edge") {
                const { panelWidth, panelHeight, marginLeft, marginTop, marginBottom } =
                    moduleHubLeftEdgeLayout(monitor)
                win.set_anchor?.(
                    Astal.WindowAnchor.LEFT |
                        Astal.WindowAnchor.TOP |
                        Astal.WindowAnchor.BOTTOM
                )
                win.set_margin_left?.(marginLeft)
                win.set_margin_right?.(0)
                win.set_margin_top?.(marginTop)
                win.set_margin_bottom?.(marginBottom)
                resizeWebViewChild(win, panelWidth, panelHeight)
            } else {
                const { cardWidth, marginLeft } = moduleHubLayout(monitor)
                win.set_anchor?.(Astal.WindowAnchor.TOP)
                win.set_margin_left?.(marginLeft)
                win.set_margin_right?.(0)
                win.set_margin_top?.(10)
                win.set_margin_bottom?.(0)
                resizeWebViewChild(win, cardWidth, SIDEBAR_HEIGHT)
            }
            break
        }
        case "dropdown": {
            const { marginTop, marginRight } = bottomRightCardLayout(monitor, DROPDOWN_HEIGHT, 8)
            win.set_anchor?.(Astal.WindowAnchor.TOP | Astal.WindowAnchor.RIGHT)
            win.set_margin_left?.(0)
            win.set_margin_right?.(marginRight)
            win.set_margin_top?.(marginTop)
            win.set_margin_bottom?.(0)
            resizeWebViewChild(win, DROPDOWN_WIDTH, DROPDOWN_HEIGHT)
            break
        }
        case "media-popup": {
            const { marginTop, marginBottom } = verticalCenterMargins(monitor, MEDIA_HEIGHT)
            win.set_anchor?.(Astal.WindowAnchor.RIGHT | Astal.WindowAnchor.TOP | Astal.WindowAnchor.BOTTOM)
            win.set_margin_left?.(0)
            win.set_margin_right?.(8)
            win.set_margin_top?.(marginTop)
            win.set_margin_bottom?.(marginBottom)
            resizeWebViewChild(win, MEDIA_WIDTH, MEDIA_HEIGHT)
            break
        }
        default:
            break
    }
}

function defaultMonitor(): Gdk.Monitor | undefined {
    const monitors = (App.monitors ?? []) as Gdk.Monitor[]
    if (monitors.length === 0) return undefined
    for (const m of monitors) {
        const primary = (m as Gdk.Monitor & { is_primary?: () => boolean }).is_primary?.()
        if (primary) return m
    }
    return monitors[0]
}

export function isHoverPanel(name: string): boolean {
    return HOVER_PANEL_NAMES.has(name)
}

export function showHoverPanel(
    name: string,
    monitor?: Gdk.Monitor,
    opts?: { moduleHubTop?: boolean }
): void {
    cancelHoverClose(name)
    const win = getWindow(name)
    if (!win) {
        print(`panel-hover: window not found: ${name}`)
        return
    }
    const mon = monitor ?? defaultMonitor()
    if (mon) {
        applyPanelLayout(name, win, mon, opts)
    }
    try {
        win.show?.()
    } catch {
        /* Gtk.ApplicationWindow vs Astal */
    }
    win.visible = true
}

export function hideHoverPanel(name: string): void {
    const win = getWindow(name)
    if (!win) return
    cancelHoverClose(name)
    win.visible = false
    try {
        win.hide?.()
    } catch {
        /* ignore */
    }
}

/** Toggle a hover panel; keyboard shortcuts use top layout for module-hub. */
export function toggleHoverPanel(
    name: string,
    monitor?: Gdk.Monitor,
    opts?: { moduleHubTop?: boolean }
): boolean {
    const win = getWindow(name)
    if (!win) return false
    if (win.visible) {
        hideHoverPanel(name)
        return false
    }
    showHoverPanel(name, monitor, opts)
    return true
}

export function cancelHoverClose(name: string): void {
    const t = timers.get(name)
    if (t != null) {
        clearTimeout(t)
        timers.delete(name)
    }
}

export function scheduleHoverClose(name: string, ms = DEFAULT_CLOSE_MS): void {
    cancelHoverClose(name)
    timers.set(
        name,
        setTimeout(() => {
            hideHoverPanel(name)
            timers.delete(name)
        }, ms)
    )
}

export function handlePanelHoverMessage(panelName: string, msg: string): void {
    if (msg === "enter") cancelHoverClose(panelName)
    else scheduleHoverClose(panelName)
}

export function registerPanelHoverHandler(
    webview: unknown,
    handlerName: string,
    panelName: string
): void {
    try {
        const wv = webview as {
            get_user_content_manager: () => {
                register_script_message_handler: (name: string, world: null) => boolean
                connect: (sig: string, cb: (mgr: unknown, val: unknown) => void) => number
            }
        }
        const mgr = wv.get_user_content_manager()
        mgr.register_script_message_handler(handlerName, null)
        mgr.connect(`script-message-received::${handlerName}`, (_m: unknown, jsVal: unknown) => {
            try {
                const msg = (jsVal as { to_string?: () => string }).to_string?.() ?? String(jsVal)
                handlePanelHoverMessage(panelName, msg)
            } catch {
                /* ignore */
            }
        })
    } catch (e) {
        print(`${handlerName} setup error: ${e}`)
    }
}

export { monitorSize, moduleHubLayout, BAR_STRIP_WIDTH_PX }
