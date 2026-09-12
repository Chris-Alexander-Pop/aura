import Gdk from "gi://Gdk?version=4.0"
import Gtk from "gi://Gtk?version=4.0"
import app from "ags/gtk4/app"
import sidecar from "./sidecar"
import { gdkMonitorGeometry, monitorTag, sanitizeMonitorTag } from "./monitor"
import { getHideShellOnFullscreen, hideHoverPanel } from "./panel-hover"
import { edgeTriggerNamesForTag } from "../widget/triggers/PanelEdgeTriggers"

type AuraWindow = Gtk.Window & {
    visible?: boolean
    hide?: () => void
    show?: () => void
    present?: () => void
    set_gdkmonitor?: (m: Gdk.Monitor) => void
    get_gdkmonitor?: () => Gdk.Monitor | null
}

const HOVER_PANELS = ["module-hub", "dropdown", "media-popup"] as const

const savedVisible = new Map<string, boolean>()
const suppressedTags = new Set<string>()

let hyprListenerAttached = false
let refreshTimer: ReturnType<typeof setTimeout> | null = null
let refreshGen = 0

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
        if (visible) {
            try {
                win.show?.()
            } catch {
                /* Gtk.ApplicationWindow vs Astal */
            }
            try {
                win.present?.()
            } catch {
                /* layer-shell */
            }
        } else {
            try {
                win.hide?.()
            } catch {
                /* ignore */
            }
        }
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

type HyprMonitorGeom = {
    id: number
    name?: string
    x: number
    y: number
    width: number
    height: number
}

function parseHyprMonitors(raw: unknown): HyprMonitorGeom[] {
    if (!Array.isArray(raw)) return []
    return raw
        .map((item) => {
            if (!item || typeof item !== "object") return null
            const o = item as Record<string, unknown>
            const id = Number(o.id)
            if (!Number.isFinite(id)) return null
            const name = typeof o.name === "string" && o.name.length > 0 ? o.name : undefined
            return {
                id,
                name,
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

function gdkMonitorsList(): Gdk.Monitor[] {
    const raw = app.monitors
    if (!raw) return []
    if (Array.isArray(raw)) return raw as Gdk.Monitor[]
    try {
        return Array.from(raw as Iterable<Gdk.Monitor>)
    } catch {
        return []
    }
}

/**
 * Map Hyprland monitor id → Aura window tag.
 * Prefer Hyprland `name` (DRM connector) — matches `monitorTag()` / `bar-wv-${tag}`.
 * Geometry matching is unreliable under fractional scale (Hypr physical vs GDK logical).
 */
function buildHyprIdToTagMap(hyprMonitors: HyprMonitorGeom[]): Map<number, string> {
    const map = new Map<number, string>()
    const gdkMonitors = gdkMonitorsList()

    for (const hm of hyprMonitors) {
        if (hm.name) {
            map.set(hm.id, sanitizeMonitorTag(hm.name))
            continue
        }
        const match = gdkMonitors.find((gm) => geometriesMatch(gdkMonitorGeometry(gm), hm))
        if (match) {
            map.set(hm.id, monitorTag(match))
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
        const isVisible = win.visible !== false
        if (isVisible || !savedVisible.has(name)) {
            if (isVisible) savedVisible.set(name, true)
            win.visible = false
            try {
                win.hide?.()
            } catch {
                /* ignore */
            }
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

function suppressMonitor(tag: string, gdkmonitor: Gdk.Monitor | null) {
    const first = !suppressedTags.has(tag)
    suppressedTags.add(tag)
    // Always re-hide — bar may have been remounted while tag stayed suppressed.
    for (const name of shellWindowNamesForTag(tag)) {
        hideWindowIfNeeded(name)
    }
    if (first && gdkmonitor) {
        hideHoverPanelsOnMonitor(gdkmonitor)
    }
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

function parseFullscreenIds(raw: unknown): number[] {
    if (!Array.isArray(raw)) return []
    const out: number[] = []
    for (const id of raw) {
        if (typeof id === "number" && Number.isFinite(id)) {
            out.push(id)
            continue
        }
        if (typeof id === "string" && id.trim() !== "") {
            const n = Number(id)
            if (Number.isFinite(n)) out.push(n)
        }
    }
    return out
}

/** Apply suppression from already-fetched fullscreen monitor ids + monitor list. */
function applyFullscreenSuppression(fullscreenIds: number[], hyprMonitors: HyprMonitorGeom[]) {
    if (!getHideShellOnFullscreen()) {
        releaseAllSuppression()
        return
    }

    const idToTag = buildHyprIdToTagMap(hyprMonitors)
    const gdkMonitors = gdkMonitorsList()

    const tagsToSuppress = new Set<string>()
    for (const id of fullscreenIds) {
        const tag = idToTag.get(id)
        if (tag) tagsToSuppress.add(tag)
    }

    for (const tag of [...suppressedTags]) {
        if (!tagsToSuppress.has(tag)) {
            releaseMonitor(tag)
        }
    }

    for (const tag of tagsToSuppress) {
        const gdkmonitor =
            gdkMonitors.find((m) => monitorTag(m) === tag) ?? gdkMonitors[0] ?? null
        suppressMonitor(tag, gdkmonitor)
    }
}

async function refreshFullscreenSuppression() {
    if (!getHideShellOnFullscreen()) {
        releaseAllSuppression()
        return
    }

    const gen = ++refreshGen
    try {
        const snapshot = await sidecar.send("Hyprland.GetBarSnapshot")
        // Drop stale responses — a newer workspace switch already applied.
        if (gen !== refreshGen) return

        const fullscreenIds = parseFullscreenIds(snapshot?.fullscreen_monitor_ids)

        let hyprMonitors = parseHyprMonitors(snapshot?.monitors)
        if (hyprMonitors.length === 0) {
            const monitorsRaw = await sidecar.send("Hyprland.GetMonitors")
            if (gen !== refreshGen) return
            hyprMonitors = parseHyprMonitors(monitorsRaw)
        }

        applyFullscreenSuppression(fullscreenIds, hyprMonitors)
    } catch (e) {
        console.error("shell-visibility: refresh failed:", e)
    }
}

/** Apply from StateChanged payload when present (no extra RPC). */
function tryApplyFromStateChanged(params: Record<string, unknown> | null | undefined): boolean {
    if (!params || typeof params !== "object") return false
    if (!("fullscreen_monitor_ids" in params)) return false
    const fullscreenIds = parseFullscreenIds(params.fullscreen_monitor_ids)
    const hyprMonitors = parseHyprMonitors(params.monitors)
    // Need monitor names to map id → bar-wv tag; fall back to RPC if missing.
    if (hyprMonitors.length === 0 || !hyprMonitors.some((m) => m.name)) return false
    applyFullscreenSuppression(fullscreenIds, hyprMonitors)
    return true
}

function scheduleRefresh(delayMs = 0) {
    if (refreshTimer != null) clearTimeout(refreshTimer)
    if (delayMs <= 0) {
        refreshTimer = null
        void refreshFullscreenSuppression()
        return
    }
    refreshTimer = setTimeout(() => {
        refreshTimer = null
        void refreshFullscreenSuppression()
    }, delayMs)
}

function attachHyprlandListener() {
    if (hyprListenerAttached) return
    hyprListenerAttached = true
    sidecar.connect(
        "notification",
        (_svc: unknown, method: string, params?: Record<string, unknown>) => {
            if (method === "Hyprland.StateChanged") {
                // Prefer inline payload — hide/show on the same event tick.
                if (tryApplyFromStateChanged(params)) return
                scheduleRefresh(0)
                return
            }
            if (method === "Hyprland.WorkspaceActive") {
                // Workspace switch: restore bar immediately even if StateChanged lags.
                scheduleRefresh(0)
                return
            }
            if (method === "Settings.Changed") {
                scheduleRefresh(0)
            }
        }
    )
}

export function initShellVisibility() {
    attachHyprlandListener()
    scheduleRefresh(0)
}

export function onHideShellSettingChanged() {
    scheduleRefresh(0)
}
