import GLib from "gi://GLib"
import Gdk from "gi://Gdk?version=4.0"
import Gtk from "gi://Gtk?version=4.0"
import app from "ags/gtk4/app"
import Bar from "../widget/bar/Bar"
import BarWebViewWindow from "../widget/webview/BarWebViewWindow"
import OSD from "../widget/osd/OSD"
import { mountEdgeTriggersForMonitor } from "./aura-settings-shell"
import { gdkMonitorConnector, gdkMonitorGeometry, monitorTag, sanitizeMonitorTag } from "./monitor"
import sidecar from "./sidecar"
import {
    edgeTriggerNamesForTag,
    rebindPanelEdgeTriggersByTag,
    unmountPanelEdgeTriggersByTag,
} from "../widget/triggers/PanelEdgeTriggers"

const USE_GTK_BAR = GLib.getenv("AURA_GTK_BAR") === "1"
const DEBUG = GLib.getenv("AURA_DEBUG_MONITOR_SHELL") === "1"

/** Tags we have mounted shell windows for. */
const mountedTags = new Set<string>()
/** Last Gdk.Monitor object bound to each tag (identity changes on hotplug). */
const mountedMonitors = new Map<string, Gdk.Monitor>()
/** Tags whose windows are mid-destroy — do not remount until settled. */
const destroyingTags = new Set<string>()

let syncTimer: ReturnType<typeof setTimeout> | null = null
let syncGen = 0
let listenerAttached = false
let hyprListenerAttached = false
/** Last Hyprland connector set we synced against (sorted, comma-joined). */
let lastHyprTagsKey = ""
/** In-flight Hyprland tag set waiting for sync (avoids debounce reset spam). */
let pendingHyprKey: string | null = null
/** Bounded retries while waiting for Gdk to catch up with Hyprland. */
let layoutRetryBudget = 0
const MAX_LAYOUT_RETRIES = 12

type AuraWindow = Gtk.Window & {
    name?: string
    set_gdkmonitor?: (m: Gdk.Monitor) => void
    destroy?: () => void
}

function shellWindowNamesForTag(tag: string): string[] {
    const names = [`bar-wv-${tag}`, `bar-flyout-${tag}`, `osd-${tag}`, ...edgeTriggerNamesForTag(tag)]
    if (app.get_window(`bar-${tag}`)) names.push(`bar-${tag}`)
    return names
}

function shellWindowsExistForTag(tag: string): boolean {
    return [`bar-wv-${tag}`, `bar-${tag}`, `osd-${tag}`].some((name) => !!app.get_window(name))
}

function destroyWindow(name: string) {
    const win = app.get_window(name) as AuraWindow | null
    if (!win) return
    // Hide + remove first so layer-shell unmaps the GdkSurface before destroy.
    // Sync destroy of a still-mapped Wayland toplevel → SIGSEGV in
    // gdk_wayland_toplevel_remove_from_session (libgtk).
    try {
        ;(win as AuraWindow & { visible?: boolean }).visible = false
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
            win.destroy?.()
        } catch {
            /* ignore */
        }
        return GLib.SOURCE_REMOVE
    })
}

function rebindMonitorShell(tag: string, monitor: Gdk.Monitor) {
    if (DEBUG) console.error(`[monitor-shell] rebind ${tag}`)
    for (const name of shellWindowNamesForTag(tag)) {
        const win = app.get_window(name) as AuraWindow | null
        if (!win) continue
        try {
            win.set_gdkmonitor?.(monitor)
        } catch (e) {
            console.error(`monitor-shell: rebind failed for ${name}:`, e)
        }
    }
    rebindPanelEdgeTriggersByTag(tag, monitor)
    mountedMonitors.set(tag, monitor)
}

function unmountMonitorShell(tag: string) {
    if (DEBUG) console.error(`[monitor-shell] unmount ${tag}`)
    destroyingTags.add(tag)
    for (const name of shellWindowNamesForTag(tag)) {
        destroyWindow(name)
    }
    unmountPanelEdgeTriggersByTag(tag)
    mountedTags.delete(tag)
    mountedMonitors.delete(tag)
    // Allow remount after unmap+idle destroy has had a chance to run.
    GLib.timeout_add(GLib.PRIORITY_DEFAULT, 200, () => {
        destroyingTags.delete(tag)
        return GLib.SOURCE_REMOVE
    })
}

/** New outputs are not ready until the DRM connector and geometry are real. */
function monitorReadyForMount(monitor: Gdk.Monitor): boolean {
    if (!gdkMonitorConnector(monitor)) return false
    const g = gdkMonitorGeometry(monitor)
    return g.width > 0 && g.height > 0
}

function mountMonitorShell(monitor: Gdk.Monitor) {
    const tag = monitorTag(monitor)
    if (mountedTags.has(tag)) return

    if (destroyingTags.has(tag)) {
        if (DEBUG) console.error(`[monitor-shell] defer mount ${tag} (destroy in flight)`)
        scheduleMonitorSync()
        return
    }

    // Leftover windows from a deferred destroy — clean up and retry once settled.
    if (shellWindowsExistForTag(tag)) {
        if (DEBUG) console.error(`[monitor-shell] defer mount ${tag} (leftover windows)`)
        for (const name of shellWindowNamesForTag(tag)) destroyWindow(name)
        unmountPanelEdgeTriggersByTag(tag)
        destroyingTags.add(tag)
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, 200, () => {
            destroyingTags.delete(tag)
            scheduleMonitorSync()
            return GLib.SOURCE_REMOVE
        })
        return
    }

    if (!monitorReadyForMount(monitor)) {
        if (DEBUG) console.error(`[monitor-shell] defer mount ${tag} (monitor not ready)`)
        scheduleMonitorSync()
        return
    }

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

/** Parse Hyprland monitor connector names into Aura window tags. */
function parseHyprMonitorTags(raw: unknown): Set<string> | null {
    if (!Array.isArray(raw)) return null
    const tags = new Set<string>()
    for (const item of raw) {
        if (!item || typeof item !== "object") continue
        const name = (item as { name?: unknown }).name
        if (typeof name === "string" && name.length > 0) {
            tags.add(sanitizeMonitorTag(name))
        }
    }
    return tags
}

function tagsKey(tags: Iterable<string>): string {
    return [...tags].sort().join(",")
}

/**
 * Tear down removed outputs and mount newly connected ones.
 * Hyprland connector names are the source of truth for which shells should exist;
 * Gdk often keeps phantom monitors after unplug.
 *
 * On real add/remove we must NOT call set_gdkmonitor on surviving shells in the
 * same turn — that races Wayland output hotplug and SIGSEGVs in GTK/WebKit.
 */
export async function syncMonitorShells() {
    const gen = ++syncGen
    const monitors = gdkMonitorsList()

    let hyprTags: Set<string> | null = null
    try {
        const raw = await sidecar.send("Hyprland.GetMonitors")
        if (gen !== syncGen) return
        hyprTags = parseHyprMonitorTags(raw)
    } catch (e) {
        if (DEBUG) console.error("[monitor-shell] Hyprland.GetMonitors failed:", e)
    }

    // Transient empty lists during hotplug — do not tear everything down.
    if (monitors.length === 0) {
        if (DEBUG) console.error("[monitor-shell] skip sync: no Gdk monitors yet")
        return
    }
    if (hyprTags && hyprTags.size === 0) {
        if (DEBUG) console.error("[monitor-shell] skip sync: no Hyprland monitors yet")
        return
    }

    const activeTags = hyprTags ?? new Set(monitors.map((m) => monitorTag(m)))
    if (hyprTags) {
        lastHyprTagsKey = tagsKey(hyprTags)
        pendingHyprKey = null
    }

    const prevMounted = new Set(mountedTags)
    const removed = [...prevMounted].filter((t) => !activeTags.has(t))
    const added = [...activeTags].filter((t) => !prevMounted.has(t))
    const layoutChanging = removed.length > 0 || added.length > 0

    if (DEBUG) {
        console.error(
            `[monitor-shell] sync active=${tagsKey(activeTags)} gdk=${monitors.map((m) => monitorTag(m)).join(",")} ` +
                `added=${added.join(",") || "-"} removed=${removed.join(",") || "-"}`
        )
    }

    for (const tag of removed) {
        unmountMonitorShell(tag)
    }

    let needRetry = false

    for (const monitor of monitors) {
        const tag = monitorTag(monitor)
        // Phantom Gdk output still listed after unplug — leave it alone.
        if (!activeTags.has(tag)) continue

        const prev = mountedMonitors.get(tag)
        if (mountedTags.has(tag) && prev === monitor) continue

        if (mountedTags.has(tag)) {
            if (layoutChanging) {
                // Survivor during add/remove: do NOT call set_gdkmonitor here —
                // it races Wayland output hotplug and SIGSEGVs in GTK/WebKit.
                // Remember the new Gdk object; a later identity-only sync can rebind.
                if (DEBUG) console.error(`[monitor-shell] skip rebind during layout change ${tag}`)
                mountedMonitors.set(tag, monitor)
            } else {
                // Pure identity churn with a stable layout — safe to rebind now.
                rebindMonitorShell(tag, monitor)
            }
            continue
        }

        if (!monitorReadyForMount(monitor)) {
            needRetry = true
            continue
        }
        mountMonitorShell(monitor)
        // mountMonitorShell may defer (destroy in flight) without adding the tag.
        if (!mountedTags.has(tag)) needRetry = true
    }

    // Hyprland already has the output but Gdk/connector is not ready yet.
    for (const tag of added) {
        if (!mountedTags.has(tag)) needRetry = true
    }

    if (needRetry) {
        if (layoutRetryBudget < MAX_LAYOUT_RETRIES) {
            layoutRetryBudget++
            scheduleMonitorSync()
        } else if (DEBUG) {
            console.error("[monitor-shell] giving up layout retries")
        }
    } else {
        layoutRetryBudget = 0
    }
}

function scheduleMonitorSync() {
    if (syncTimer != null) clearTimeout(syncTimer)
    // Hyprland/Gdk may emit remove+add in quick succession; wait for layout to settle
    // before creating WebKit layer surfaces on a newly attached output.
    syncTimer = setTimeout(() => {
        syncTimer = null
        void syncMonitorShells()
    }, 500)
}

function onHyprlandMonitorsPayload(params?: Record<string, unknown>) {
    if (!params || !Array.isArray(params.monitors)) return
    const tags = parseHyprMonitorTags(params.monitors)
    if (!tags || tags.size === 0) return
    const key = tagsKey(tags)
    if (key === lastHyprTagsKey) return
    if (key === pendingHyprKey) return
    if (DEBUG) console.error(`[monitor-shell] Hyprland monitors changed → ${key}`)
    pendingHyprKey = key
    scheduleMonitorSync()
}

function attachHyprlandListener() {
    if (hyprListenerAttached) return
    hyprListenerAttached = true
    sidecar.connect(
        "notification",
        (_svc: unknown, method: string, params?: Record<string, unknown>) => {
            if (method === "Hyprland.StateChanged") {
                onHyprlandMonitorsPayload(params)
            }
        }
    )
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
    void syncMonitorShells()
    attachMonitorListener()
    attachHyprlandListener()
}
