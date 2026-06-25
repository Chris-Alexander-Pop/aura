import App from "ags/gtk4/app"

const DEFAULT_CLOSE_MS = 450

type AuraWindow = {
    visible?: boolean
    show?: () => void
    hide?: () => void
}

const timers = new Map<string, ReturnType<typeof setTimeout>>()

function getWindow(name: string): AuraWindow | null {
    return App.get_window(name) as AuraWindow | null
}

export function showHoverPanel(name: string): void {
    cancelHoverClose(name)
    const win = getWindow(name)
    if (!win) {
        print(`panel-hover: window not found: ${name}`)
        return
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
    win.visible = false
    try {
        win.hide?.()
    } catch {
        /* ignore */
    }
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
