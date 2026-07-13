import Gdk from "gi://Gdk?version=4.0"
import Gtk from "gi://Gtk?version=4.0"
import app from "ags/gtk4/app"
import sidecar from "./sidecar"
import { gdkMonitorGeometry, monitorTag } from "./monitor"
import { getHideShellOnFullscreen, hideHoverPanel } from "./panel-hover"
import { edgeTriggerNamesForTag } from "../widget/triggers/PanelEdgeTriggers"

type AuraWindow = Gtk.Window & {
    visible?: boolean
    set_gdkmonitor?: (m: Gdk.Monitor) => void
    get_gdkmonitor?: () => Gdk.Monitor | null
}

const HOVER_PANELS = ["module-hub", "dropdown", "media-popup"] as const

const savedVisible = new Map<string, boolean>()
const suppressedTags = new Set<string>()

let hyprListenerAttached = false
let refreshTimer: ReturnType<typeof setTimeout> | null = null

function getWindow(name: string): AuraWindow | null {
    return app.get_window(name) as AuraWindow | null
}

function setWindowVisible(name: string, visible: boolean) {
    const win = getWindow(name)
    if (!win) return
    try {
        // Skip no-ops — flipping visible on a torn-down layer surface crashes GTK.
        if (win.visible === visible) return
        win.visible = visible
    } catch (e) {
        console.error(`shell-visibility: set visible=${visible} failed for ${name}:`, e)
    }
}

function shellWindowNamesForTag(tag: string): string[] {
    const names = [`bar-wv-${tag}`, `bar-flyout-${tag}`, `osd-${tag}`, ...edgeTriggerNamesForTag(tag)]
    const gtkBar = getWindow(`bar-${tag}`)
    if (gtkBar) names.push(`bar-${tag}`)
    return names
}

type HyprMonitorGeom = { id: number; x: number; y: number; width: number; height: number }

function parseHyprMonitors(raw: unknown): HyprMonitorGeom[] {
    if (!Array.isArray(raw)) return []
    return raw
        .map((item) => {
            if (!item || typeof item !== "object") return null
            const o = item as Record<string, unknown>
            const id = Number(o.id)
            if (!Number.isFinite(id)) return null
            return {
                id,
                x: Number(o.x) || 0,
                y: Number(o.y) || 0,
                width: Number(o.width) || 0,
                height: Number(o.height) || 0,
            }
        })
        .filter((m): m is HyprMonitorGeom => m != null)
}

function geometriesMatch(
    a: { x: number; y: number; width: number; height: number },
    b: { x: number; y: number; width: number; height: number }
): boolean {
    return a.x === b.x && a.y === b.y && a.width === b.width && a.height === b.height
}

function buildHyprIdToTagMap(hyprMonitors: HyprMonitorGeom[]): Map<number, string> {
    const map = new Map<number, string>()
    const gdkMonitors = (app.monitors || []) as Gdk.Monitor[]
    const usedTags = new Set<string>()

    for (const hm of hyprMonitors) {
        const match = gdkMonitors.find((gm) => {
            const g = gdkMonitorGeometry(gm)
            return geometriesMatch(g, hm)
        })
        if (match) {
            const tag = monitorTag(match)
            map.set(hm.id, tag)
            usedTags.add(tag)
        }
    }

    const sortedHypr = [...hyprMonitors].sort((a, b) => a.id - b.id)
    const sortedGdk = [...gdkMonitors].sort((a, b) => monitorTag(a).localeCompare(monitorTag(b)))
    for (let i = 0; i < sortedHypr.length; i++) {
        const hm = sortedHypr[i]
        if (map.has(hm.id)) continue
        const gm = sortedGdk[i]
        if (gm) {
            map.set(hm.id, monitorTag(gm))
        }
    }

    return map
}

function hideWindowIfNeeded(name: string) {
    const win = getWindow(name)
    if (!win) return
    try {
        if (win.visible) {
            savedVisible.set(name, true)
            win.visible = false
        }
    } catch (e) {
        console.error(`shell-visibility: hide failed for ${name}:`, e)
    }
}

function restoreWindowIfSaved(name: string) {
    if (!savedVisible.has(name)) return
    const wasVisible = savedVisible.get(name)
    savedVisible.delete(name)
    if (wasVisible) {
        setWindowVisible(name, true)
    }
}

function hideHoverPanelsOnMonitor(gdkmonitor: Gdk.Monitor) {
    for (const panelName of HOVER_PANELS) {
        const win = getWindow(panelName)
        if (!win?.visible) continue
        const panelMon = win.get_gdkmonitor?.()
        if (panelMon && panelMon === gdkmonitor) {
            hideHoverPanel(panelName)
        }
    }
}

function suppressMonitor(tag: string, gdkmonitor: Gdk.Monitor) {
    if (suppressedTags.has(tag)) return
    suppressedTags.add(tag)
    for (const name of shellWindowNamesForTag(tag)) {
        hideWindowIfNeeded(name)
    }
    hideHoverPanelsOnMonitor(gdkmonitor)
}

function releaseMonitor(tag: string) {
    if (!suppressedTags.has(tag)) return
    suppressedTags.delete(tag)
    for (const name of shellWindowNamesForTag(tag)) {
        restoreWindowIfSaved(name)
    }
}

function releaseAllSuppression() {
    const tags = [...suppressedTags]
    for (const tag of tags) {
        releaseMonitor(tag)
    }
    savedVisible.clear()
}

async function refreshFullscreenSuppression() {
    if (!getHideShellOnFullscreen()) {
        releaseAllSuppression()
        return
    }

    try {
        const [snapshot, monitorsRaw] = await Promise.all([
            sidecar.send("Hyprland.GetBarSnapshot"),
            sidecar.send("Hyprland.GetMonitors"),
        ])

        const fullscreenIds: number[] = Array.isArray(snapshot?.fullscreen_monitor_ids)
            ? snapshot.fullscreen_monitor_ids.filter((id: unknown) => typeof id === "number")
            : []

        const hyprMonitors = parseHyprMonitors(monitorsRaw)
        const idToTag = buildHyprIdToTagMap(hyprMonitors)
        const gdkMonitors = (app.monitors || []) as Gdk.Monitor[]

        const tagsToSuppress = new Set<string>()
        for (const id of fullscreenIds) {
            const tag = idToTag.get(id)
            if (tag) tagsToSuppress.add(tag)
        }

        for (const tag of suppressedTags) {
            if (!tagsToSuppress.has(tag)) {
                releaseMonitor(tag)
            }
        }

        for (const tag of tagsToSuppress) {
            const gdkmonitor =
                gdkMonitors.find((m) => monitorTag(m) === tag) ?? gdkMonitors[0]
            if (gdkmonitor) {
                suppressMonitor(tag, gdkmonitor)
            }
        }
    } catch (e) {
        console.error("shell-visibility: refresh failed:", e)
    }
}

function scheduleRefresh() {
    if (refreshTimer != null) clearTimeout(refreshTimer)
    refreshTimer = setTimeout(() => {
        refreshTimer = null
        void refreshFullscreenSuppression()
    }, 50)
}

function attachHyprlandListener() {
    if (hyprListenerAttached) return
    hyprListenerAttached = true
    sidecar.connect("notification", (_svc: unknown, method: string) => {
        if (method === "Hyprland.StateChanged" || method === "Settings.Changed") {
            scheduleRefresh()
        }
    })
}

export function initShellVisibility() {
    attachHyprlandListener()
    scheduleRefresh()
}

export function onHideShellSettingChanged() {
    scheduleRefresh()
}
