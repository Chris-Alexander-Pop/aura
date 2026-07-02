import GLib from "gi://GLib"
import Gdk from "gi://Gdk?version=4.0"
import Gtk from "gi://Gtk?version=4.0"
import app from "ags/gtk4/app"
import Bar from "../widget/bar/Bar"
import BarWebViewWindow from "../widget/webview/BarWebViewWindow"
import OSD from "../widget/osd/OSD"
import { mountEdgeTriggersForMonitor } from "./aura-settings-shell"
import { monitorTag } from "./monitor"
import {
    edgeTriggerNamesForTag,
    unmountPanelEdgeTriggersByTag,
} from "../widget/triggers/PanelEdgeTriggers"

const USE_GTK_BAR = GLib.getenv("AURA_GTK_BAR") === "1"
const DEBUG = GLib.getenv("AURA_DEBUG_MONITOR_SHELL") === "1"

/** Tags we have mounted shell windows for. */
const mountedTags = new Set<string>()
/** Last Gdk.Monitor object bound to each tag (identity changes on hotplug). */
const mountedMonitors = new Map<string, Gdk.Monitor>()

let syncTimer: ReturnType<typeof setTimeout> | null = null
let listenerAttached = false

type AuraWindow = Gtk.Window & {
    set_gdkmonitor?: (m: Gdk.Monitor) => void
    destroy?: () => void
}

function shellWindowNamesForTag(tag: string): string[] {
    const names = [`bar-wv-${tag}`, `bar-flyout-${tag}`, `osd-${tag}`, ...edgeTriggerNamesForTag(tag)]
    if (app.get_window(`bar-${tag}`)) names.push(`bar-${tag}`)
    return names
}

function destroyWindow(name: string) {
    const win = app.get_window(name) as AuraWindow | null
    if (!win) return
    try {
        app.remove_window(win)
    } catch {
        /* ignore */
    }
    try {
        win.destroy?.()
    } catch {
        /* ignore */
    }
}

function unmountMonitorShell(tag: string) {
    if (DEBUG) console.error(`[monitor-shell] unmount ${tag}`)
    for (const name of shellWindowNamesForTag(tag)) {
        destroyWindow(name)
    }
    unmountPanelEdgeTriggersByTag(tag)
    mountedTags.delete(tag)
    mountedMonitors.delete(tag)
}

function mountMonitorShell(monitor: Gdk.Monitor) {
    const tag = monitorTag(monitor)
    if (mountedTags.has(tag)) return

    if (DEBUG) console.error(`[monitor-shell] mount ${tag}`)

    try {
        if (USE_GTK_BAR) {
            Bar(monitor)
        } else {
            BarWebViewWindow(monitor)
        }
    } catch (e) {
        console.error(`Failed to create bar for monitor ${tag}:`, e)
    }

    try {
        mountEdgeTriggersForMonitor(monitor)
    } catch (e) {
        console.error(`Failed to create edge triggers for monitor ${tag}:`, e)
    }

    try {
        OSD(monitor)
    } catch (e) {
        console.error(`Failed to create OSD for monitor ${tag}:`, e)
    }

    mountedTags.add(tag)
    mountedMonitors.set(tag, monitor)
}

/** Tear down removed outputs and mount newly connected ones. */
export function syncMonitorShells() {
    const monitors = (app.monitors || []) as Gdk.Monitor[]
    const activeTags = new Set<string>()

    for (const monitor of monitors) {
        activeTags.add(monitorTag(monitor))
    }

    for (const tag of [...mountedTags]) {
        if (!activeTags.has(tag)) {
            unmountMonitorShell(tag)
        }
    }

    for (const monitor of monitors) {
        const tag = monitorTag(monitor)
        const prev = mountedMonitors.get(tag)
        if (mountedTags.has(tag) && prev === monitor) continue

        if (mountedTags.has(tag)) {
            unmountMonitorShell(tag)
        }
        mountMonitorShell(monitor)
    }
}

function scheduleMonitorSync() {
    if (syncTimer != null) clearTimeout(syncTimer)
    // Hyprland/Gdk may emit remove+add in quick succession; wait for layout to settle.
    syncTimer = setTimeout(() => {
        syncTimer = null
        syncMonitorShells()
    }, 300)
}

function attachMonitorListener() {
    if (listenerAttached) return
    listenerAttached = true

    const display = Gdk.Display.get_default()
    if (!display) {
        console.error("monitor-shell: no Gdk display")
        return
    }

    display.get_monitors().connect("items-changed", () => {
        if (DEBUG) console.error("[monitor-shell] Gdk monitors items-changed")
        scheduleMonitorSync()
    })
}

/** Mount shell UI for every current output and keep it in sync on hotplug. */
export function initMonitorShell() {
    syncMonitorShells()
    attachMonitorListener()
}
