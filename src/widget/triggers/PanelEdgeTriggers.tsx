import GLib from "gi://GLib"
import Gtk from "gi://Gtk?version=4.0"
import Astal from "gi://Astal?version=4.0"
import Gdk from "gi://Gdk?version=4.0"
import app from "ags/gtk4/app"
import { cancelHoverClose, scheduleHoverClose, showHoverPanel } from "../../lib/panel-hover"
import { moduleHubLayout, verticalCenterMargins, monitorSize } from "../../lib/monitor"

/** Keep strong refs so GJS does not collect layer-shell windows. */
export const edgeTriggerWindows: Gtk.Window[] = []

const TRIGGER_SIZE = 2
const MEDIA_TRIGGER_H = 140
/** L-shaped dropdown corner: length of each leg along bottom and right edges. */
const DROPDOWN_CORNER_LEG = 56

const DEBUG_TRIGGERS = GLib.getenv("AURA_DEBUG_EDGE_TRIGGERS") === "1"
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
    applyCss(
        box,
        `background-color: ${HIT_RGBA}; min-width: ${spec.width > 0 ? spec.width : 0}px; min-height: ${spec.height > 0 ? spec.height : 0}px;`
    )
    applyCss(win, `background-color: ${WIN_RGBA};`)

    const motion = Gtk.EventControllerMotion.new()
    motion.connect("enter", () => {
        if (DEBUG_TRIGGERS) {
            console.error(`edge-trigger enter: ${spec.name} → ${spec.targetWindow}`)
        }
        cancelHoverClose(spec.targetWindow)
        showHoverPanel(spec.targetWindow, gdkmonitor)
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

/** Bottom-right L: short strip along the bottom edge + short strip up the right edge. */
function mountDropdownCornerTriggers(gdkmonitor: Gdk.Monitor, tag: string) {
    const { height } = monitorSize(gdkmonitor)
    const target = "dropdown"

    // Horizontal leg — flush with bottom-right corner, extends left
    createEdgeTrigger(gdkmonitor, {
        name: `dropdown-trigger-bottom-${tag}`,
        targetWindow: target,
        anchor: Astal.WindowAnchor.TOP | Astal.WindowAnchor.RIGHT,
        marginRight: 0,
        marginTop: height - TRIGGER_SIZE,
        width: DROPDOWN_CORNER_LEG,
        height: TRIGGER_SIZE,
    })

    // Vertical leg — flush with bottom-right corner, extends up
    createEdgeTrigger(gdkmonitor, {
        name: `dropdown-trigger-right-${tag}`,
        targetWindow: target,
        anchor: Astal.WindowAnchor.TOP | Astal.WindowAnchor.RIGHT,
        marginRight: 0,
        marginTop: height - DROPDOWN_CORNER_LEG,
        width: TRIGGER_SIZE,
        height: DROPDOWN_CORNER_LEG,
    })
}

/** Imperative layer-shell strips — must call app.add_window (JSX alone was not showing). */
export function mountPanelEdgeTriggers(gdkmonitor: Gdk.Monitor) {
    const tag = monitorTag(gdkmonitor)
    const { cardWidth, marginLeft } = moduleHubLayout(gdkmonitor)
    const mediaMargins = verticalCenterMargins(gdkmonitor, MEDIA_TRIGGER_H)

    if (DEBUG_TRIGGERS) {
        console.error(`mountPanelEdgeTriggers: monitor=${tag} module-hub w=${cardWidth} ml=${marginLeft}`)
    }

    createEdgeTrigger(gdkmonitor, {
        name: `module-hub-trigger-${tag}`,
        targetWindow: "module-hub",
        anchor: Astal.WindowAnchor.TOP,
        marginLeft,
        marginTop: 0,
        width: cardWidth,
        height: TRIGGER_SIZE,
    })

    mountDropdownCornerTriggers(gdkmonitor, tag)

    createEdgeTrigger(gdkmonitor, {
        name: `media-popup-trigger-${tag}`,
        targetWindow: "media-popup",
        anchor: Astal.WindowAnchor.RIGHT | Astal.WindowAnchor.TOP | Astal.WindowAnchor.BOTTOM,
        marginRight: 0,
        marginTop: mediaMargins.marginTop,
        marginBottom: mediaMargins.marginBottom,
        width: TRIGGER_SIZE,
        height: MEDIA_TRIGGER_H,
    })
}
