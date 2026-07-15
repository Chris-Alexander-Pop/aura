import GLib from "gi://GLib"
import Gtk from "gi://Gtk?version=4.0"
import Astal from "gi://Astal?version=4.0"
import Gdk from "gi://Gdk?version=4.0"
import app from "ags/gtk4/app"
import { cancelHoverClose, scheduleHoverClose, showHoverPanel, MODULE_HUB_ENABLED } from "../../lib/panel-hover"
import {
    BAR_STRIP_WIDTH_PX,
    moduleHubLayout,
    verticalCenterMargins,
    monitorSize,
    monitorTag,
} from "../../lib/monitor"

export type ModuleHubTriggerMode = "left_edge" | "top_third" | "none"

/** Keep strong refs so GJS does not collect layer-shell windows. */
export const edgeTriggerWindows: Gtk.Window[] = []

const triggersByMonitor = new Map<string, Gtk.Window[]>()

const TRIGGER_SIZE = 2
/** Top module-hub strip — wider than corner legs so hover is reachable. */
const TOP_HUB_TRIGGER_H = 10
const MEDIA_TRIGGER_H = 140
/** L-shaped dropdown corner: length of each leg along bottom and right edges. */
const DROPDOWN_CORNER_LEG = 56

const DEBUG_TRIGGERS = GLib.getenv("AURA_DEBUG_EDGE_TRIGGERS") === "1"
const HIT_RGBA = DEBUG_TRIGGERS ? "rgba(255, 34, 34, 0.92)" : "rgba(255, 255, 255, 0.01)"
const WIN_RGBA = DEBUG_TRIGGERS ? "rgba(255, 0, 0, 0.25)" : "transparent"

export { monitorTag }

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
    layer?: number
}

function removeFromEdgeRefs(win: Gtk.Window) {
    const idx = edgeTriggerWindows.indexOf(win)
    if (idx !== -1) edgeTriggerWindows.splice(idx, 1)
}

function destroyTriggerWindow(win: Gtk.Window) {
    removeFromEdgeRefs(win)
    // Unmap before destroy — sync destroy of mapped layer surfaces SIGSEGVs
    // in gdk_wayland_toplevel_remove_from_session.
    try {
        win.visible = false
    } catch {
        /* ignore */
    }
    try {
        app.remove_window(win)
    } catch {
        /* ignore */
    }
    GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {
        try {
            win.destroy()
        } catch {
            /* ignore */
        }
        return GLib.SOURCE_REMOVE
    })
}

function createEdgeTrigger(gdkmonitor: Gdk.Monitor, spec: TriggerSpec) {
    const win = new Astal.Window({
        name: spec.name,
        visible: true,
        anchor: spec.anchor,
        layer: spec.layer ?? Astal.Layer.OVERLAY,
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

function mountModuleHubTrigger(
    gdkmonitor: Gdk.Monitor,
    tag: string,
    mode: ModuleHubTriggerMode
): Gtk.Window | null {
    if (!MODULE_HUB_ENABLED || mode === "none") return null

    if (mode === "left_edge") {
        const { height } = monitorSize(gdkmonitor)
        // Sit just right of the exclusive bar strip — a 2px strip at x=0 is under the bar.
        return createEdgeTrigger(gdkmonitor, {
            name: `module-hub-trigger-left-${tag}`,
            targetWindow: "module-hub",
            anchor: Astal.WindowAnchor.LEFT | Astal.WindowAnchor.TOP,
            marginLeft: BAR_STRIP_WIDTH_PX,
            marginTop: 0,
            width: TRIGGER_SIZE,
            height,
        })
    }

    const { cardWidth, marginLeft } = moduleHubLayout(gdkmonitor)
    return createEdgeTrigger(gdkmonitor, {
        name: `module-hub-trigger-${tag}`,
        targetWindow: "module-hub",
        anchor: Astal.WindowAnchor.TOP,
        marginLeft,
        marginTop: 0,
        width: cardWidth,
        height: TOP_HUB_TRIGGER_H,
    })
}

/** Bottom-right L: short strip along the bottom edge + short strip up the right edge. */
function mountDropdownCornerTriggers(gdkmonitor: Gdk.Monitor, tag: string): Gtk.Window[] {
    const { height } = monitorSize(gdkmonitor)
    const target = "dropdown"
    const out: Gtk.Window[] = []

    out.push(
        createEdgeTrigger(gdkmonitor, {
            name: `dropdown-trigger-bottom-${tag}`,
            targetWindow: target,
            anchor: Astal.WindowAnchor.TOP | Astal.WindowAnchor.RIGHT,
            marginRight: 0,
            marginTop: height - TRIGGER_SIZE,
            width: DROPDOWN_CORNER_LEG,
            height: TRIGGER_SIZE,
            layer: Astal.Layer.TOP,
        })
    )

    out.push(
        createEdgeTrigger(gdkmonitor, {
            name: `dropdown-trigger-right-${tag}`,
            targetWindow: target,
            anchor: Astal.WindowAnchor.TOP | Astal.WindowAnchor.RIGHT,
            marginRight: 0,
            marginTop: height - DROPDOWN_CORNER_LEG,
            width: TRIGGER_SIZE,
            height: DROPDOWN_CORNER_LEG,
            layer: Astal.Layer.TOP,
        })
    )

    return out
}

/** Remove all edge triggers for a monitor tag (dropdown, media, module-hub). */
export function unmountPanelEdgeTriggersByTag(tag: string) {
    const wins = triggersByMonitor.get(tag)
    if (!wins) return
    for (const win of wins) {
        destroyTriggerWindow(win)
    }
    triggersByMonitor.delete(tag)
}

/** Remove all edge triggers for a monitor (dropdown, media, module-hub). */
export function unmountPanelEdgeTriggers(gdkmonitor: Gdk.Monitor) {
    unmountPanelEdgeTriggersByTag(monitorTag(gdkmonitor))
}

/** Re-point existing edge triggers at a new Gdk.Monitor instance (hotplug identity churn). */
export function rebindPanelEdgeTriggersByTag(tag: string, gdkmonitor: Gdk.Monitor) {
    const wins = triggersByMonitor.get(tag)
    if (!wins) return
    for (const win of wins) {
        try {
            const w = win as Gtk.Window & { set_gdkmonitor?: (m: Gdk.Monitor) => void }
            w.set_gdkmonitor?.(gdkmonitor)
        } catch {
            /* ignore */
        }
    }
}

/** Imperative layer-shell strips — must call app.add_window (JSX alone was not showing). */
export function mountPanelEdgeTriggers(
    gdkmonitor: Gdk.Monitor,
    moduleHubMode: ModuleHubTriggerMode = "top_third"
) {
    const tag = monitorTag(gdkmonitor)
    unmountPanelEdgeTriggers(gdkmonitor)

    const mediaMargins = verticalCenterMargins(gdkmonitor, MEDIA_TRIGGER_H)
    const created: Gtk.Window[] = []

    if (DEBUG_TRIGGERS) {
        console.error(`mountPanelEdgeTriggers: monitor=${tag} module-hub mode=${moduleHubMode}`)
    }

    const hubTrigger = mountModuleHubTrigger(gdkmonitor, tag, moduleHubMode)
    if (hubTrigger) created.push(hubTrigger)

    created.push(...mountDropdownCornerTriggers(gdkmonitor, tag))

    created.push(
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
    )

    triggersByMonitor.set(tag, created)
}

export function remountPanelEdgeTriggers(
    gdkmonitor: Gdk.Monitor,
    moduleHubMode: ModuleHubTriggerMode
) {
    mountPanelEdgeTriggers(gdkmonitor, moduleHubMode)
}

/** Window name prefixes for edge triggers on a monitor tag. */
export function edgeTriggerNamesForTag(tag: string): string[] {
    return [
        `module-hub-trigger-${tag}`,
        `module-hub-trigger-left-${tag}`,
        `dropdown-trigger-bottom-${tag}`,
        `dropdown-trigger-right-${tag}`,
        `media-popup-trigger-${tag}`,
    ]
}
