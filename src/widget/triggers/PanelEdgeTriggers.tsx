import GLib from "gi://GLib"
import Gtk from "gi://Gtk?version=4.0"
import Astal from "gi://Astal?version=4.0"
import Gdk from "gi://Gdk?version=4.0"
import app from "ags/gtk4/app"
import { cancelHoverClose, scheduleHoverClose, showHoverPanel } from "../../lib/panel-hover"

/** Keep strong refs so GJS does not collect layer-shell windows. */
export const edgeTriggerWindows: Gtk.Window[] = []

export const BAR_STRIP_W = 56
const TRIGGER_W = 24
const CAL_TRIGGER_H = 140

const DEBUG_TRIGGERS = GLib.getenv("AURA_DEBUG_EDGE_TRIGGERS") !== "0"
const HIT_RGBA = DEBUG_TRIGGERS ? "rgba(255, 34, 34, 0.92)" : "rgba(255, 255, 255, 0.01)"
const WIN_RGBA = DEBUG_TRIGGERS ? "rgba(255, 0, 0, 0.25)" : "transparent"

function monitorTag(monitor: Gdk.Monitor): string {
    return String(monitor.model ?? "monitor").replace(/[^a-zA-Z0-9_-]/g, "-")
}

function applyCss(widget: Gtk.Widget, css: string) {
    const provider = new Gtk.CssProvider()
    provider.load_from_string(`* { ${css} }`)
    widget.get_style_context().add_provider(provider, Gtk.STYLE_PROVIDER_PRIORITY_USER)
}

type TriggerSpec = {
    name: string
    targetWindow: string
    anchor: number
    marginLeft?: number
    marginRight?: number
    marginTop?: number
    marginBottom?: number
    width: number
    height: number
}

function createEdgeTrigger(gdkmonitor: Gdk.Monitor, spec: TriggerSpec) {
    const win = new Astal.Window({
        name: spec.name,
        visible: true,
        anchor: spec.anchor,
        layer: Astal.Layer.OVERLAY,
        exclusivity: Astal.Exclusivity.IGNORE,
        gdkmonitor,
    })
    win.set_gdkmonitor(gdkmonitor)
    win.set_margin_left(spec.marginLeft ?? 0)
    win.set_margin_right(spec.marginRight ?? 0)
    win.set_margin_top(spec.marginTop ?? 0)
    win.set_margin_bottom(spec.marginBottom ?? 0)

    const box = new Gtk.Box()
    box.set_size_request(spec.width, spec.height)
    box.set_vexpand(spec.height < 0)
    box.set_hexpand(spec.width < 0)
    applyCss(box, `background-color: ${HIT_RGBA}; min-width: ${spec.width > 0 ? spec.width : 0}px; min-height: ${spec.height > 0 ? spec.height : 0}px;`)
    applyCss(win, `background-color: ${WIN_RGBA};`)

    const motion = Gtk.EventControllerMotion.new()
    motion.connect("enter", () => {
        if (DEBUG_TRIGGERS) {
            console.error(`edge-trigger enter: ${spec.name} → ${spec.targetWindow}`)
        }
        cancelHoverClose(spec.targetWindow)
        showHoverPanel(spec.targetWindow)
    })
    motion.connect("leave", () => {
        if (DEBUG_TRIGGERS) {
            console.error(`edge-trigger leave: ${spec.name} → ${spec.targetWindow}`)
        }
        scheduleHoverClose(spec.targetWindow)
    })
    box.add_controller(motion)

    win.set_child(box)
    app.add_window(win)
    edgeTriggerWindows.push(win)

    if (DEBUG_TRIGGERS) {
        console.error(`edge-trigger registered: ${spec.name} on ${monitorTag(gdkmonitor)}`)
    }

    return win
}

/** Imperative layer-shell strips — must call app.add_window (JSX alone was not showing). */
export function mountPanelEdgeTriggers(gdkmonitor: Gdk.Monitor) {
    const tag = monitorTag(gdkmonitor)
    console.error(`mountPanelEdgeTriggers: monitor=${tag}`)

    createEdgeTrigger(gdkmonitor, {
        name: `dropdown-trigger-${tag}`,
        targetWindow: "dropdown",
        anchor:
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.LEFT |
            Astal.WindowAnchor.RIGHT,
        marginTop: 0,
        width: -1,
        height: TRIGGER_W,
    })

    createEdgeTrigger(gdkmonitor, {
        name: `calendar-trigger-${tag}`,
        targetWindow: "calendar",
        anchor: Astal.WindowAnchor.LEFT | Astal.WindowAnchor.TOP,
        marginLeft: BAR_STRIP_W,
        marginTop: 8,
        width: TRIGGER_W,
        height: CAL_TRIGGER_H,
    })

    createEdgeTrigger(gdkmonitor, {
        name: `sidebar-trigger-${tag}`,
        targetWindow: "sidebar",
        anchor:
            Astal.WindowAnchor.LEFT |
            Astal.WindowAnchor.TOP |
            Astal.WindowAnchor.BOTTOM,
        marginLeft: BAR_STRIP_W,
        marginTop: 8 + CAL_TRIGGER_H + 4,
        width: TRIGGER_W,
        height: -1,
    })
}
