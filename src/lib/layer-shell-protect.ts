import GLib from "gi://GLib"
import Gtk from "gi://Gtk?version=4.0"
// @ts-ignore — GIR has no TS types in this tree
import Gtk4LayerShell from "gi://Gtk4LayerShell?version=1.0"

type LayerWindow = Gtk.Window & {
    name?: string
    visible?: boolean
    get_child?: () => Gtk.Widget | null
    set_child?: (child: Gtk.Widget | null) => void
}

type WebViewChild = Gtk.Widget & {
    terminate_web_process?: () => void
    stop_loading?: () => void
    load_uri?: (uri: string) => void
}

type ClosedHandler = (win: Gtk.Window) => void

const armed = new WeakSet<Gtk.Window>()
let onLayerClosed: ClosedHandler | null = null

/** monitor-shell registers this so compositor `.closed` can freeze that output's shell. */
export function setLayerClosedHandler(handler: ClosedHandler | null): void {
    onLayerClosed = handler
}

function stopWebViewChild(win: LayerWindow): void {
    try {
        const child = win.get_child?.() as WebViewChild | null
        if (!child) return
        try {
            child.terminate_web_process?.()
        } catch {
            /* ignore */
        }
        try {
            child.stop_loading?.()
        } catch {
            /* ignore */
        }
        try {
            child.load_uri?.("about:blank")
        } catch {
            /* ignore */
        }
    } catch {
        /* ignore */
    }
}

/**
 * Stop WebKit immediately, then unmap on idle.
 * Do not query GdkSurface / mapped here: gtk4-layer-shell `.closed` runs
 * during Wayland dispatch and that SIGSEGVs (gdk_surface_get_display).
 */
export function quiesceLayerWindow(win: Gtk.Window): void {
    const w = win as LayerWindow
    stopWebViewChild(w)
    GLib.idle_add(GLib.PRIORITY_HIGH_IDLE, () => {
        try {
            w.visible = false
        } catch {
            /* surface already gone */
        }
        try {
            w.set_child?.(null)
        } catch {
            /* ignore */
        }
        return GLib.SOURCE_REMOVE
    })
}

/**
 * gtk4-layer-shell 1.3+ ignores `zwlr_layer_surface_v1.closed` unless
 * respect_close is on. Without this, an unplugged output leaves the window
 * mapped on a dead GdkSurface and the next WebKit/GSK frame SIGSEGVs.
 *
 * close-request must return true so GTK does not destroy the window on the
 * Wayland event (that path is also a SIGSEGV).
 */
export function protectLayerShellWindow(win: Gtk.Window): void {
    if (armed.has(win)) return
    armed.add(win)

    try {
        Gtk4LayerShell.set_respect_close(win, true)
    } catch {
        /* older gtk4-layer-shell */
    }

    win.connect("close-request", () => {
        stopWebViewChild(win as LayerWindow)
        try {
            onLayerClosed?.(win)
        } catch (e) {
            console.error("layer-shell-protect: closed handler failed:", e)
        }
        return true
    })
}
