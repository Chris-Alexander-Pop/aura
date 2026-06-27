import Gdk from "gi://Gdk?version=4.0"
import app from "ags/gtk4/app"
import sidecar from "./sidecar"
import {
    mountPanelEdgeTriggers,
    type ModuleHubTriggerMode,
} from "../widget/triggers/PanelEdgeTriggers"
import { setModuleHubTriggerMode, setHideShellOnFullscreen } from "./panel-hover"
import { onHideShellSettingChanged } from "./shell-visibility"

export type { ModuleHubTriggerMode }

let cachedModuleHubTrigger: ModuleHubTriggerMode = "top_third"
let cachedHideShellOnFullscreen = true
let settingsListenerAttached = false

export function getModuleHubTriggerMode(): ModuleHubTriggerMode {
    return cachedModuleHubTrigger
}

export function getHideShellOnFullscreen(): boolean {
    return cachedHideShellOnFullscreen
}

function parseModuleHubTrigger(raw: unknown): ModuleHubTriggerMode {
    if (raw === "left_edge" || raw === "top_third" || raw === "none") return raw
    return "top_third"
}

function applyCachedSettings(settings: {
    module_hub_trigger?: unknown
    hide_shell_on_fullscreen?: unknown
}) {
    const prevTrigger = cachedModuleHubTrigger
    cachedModuleHubTrigger = parseModuleHubTrigger(settings.module_hub_trigger)
    cachedHideShellOnFullscreen =
        typeof settings.hide_shell_on_fullscreen === "boolean"
            ? settings.hide_shell_on_fullscreen
            : true

    setModuleHubTriggerMode(cachedModuleHubTrigger)
    setHideShellOnFullscreen(cachedHideShellOnFullscreen)
    onHideShellSettingChanged()

    if (prevTrigger !== cachedModuleHubTrigger) {
        remountAllEdgeTriggers(cachedModuleHubTrigger)
    }
}

function remountAllEdgeTriggers(mode: ModuleHubTriggerMode) {
    const monitors = app.monitors || []
    for (const monitor of monitors) {
        try {
            mountPanelEdgeTriggers(monitor as Gdk.Monitor, mode)
        } catch (e) {
            console.error("Failed to remount edge triggers:", e)
        }
    }
}

async function refreshSettingsFromSidecar() {
    try {
        const result = await sidecar.send("Settings.Get")
        const settings = result?.settings
        if (settings && typeof settings === "object") {
            applyCachedSettings(settings as Record<string, unknown>)
        }
    } catch (e) {
        console.error("aura-settings-shell: Settings.Get failed:", e)
    }
}

function attachSettingsListener() {
    if (settingsListenerAttached) return
    settingsListenerAttached = true
    sidecar.connect("notification", (_svc: unknown, method: string) => {
        if (method === "Settings.Changed") {
            void refreshSettingsFromSidecar()
        }
    })
}

/** Load settings and wire Settings.Changed → trigger remount. */
export function initAuraSettingsShell() {
    attachSettingsListener()
    void refreshSettingsFromSidecar()
}

/** Mount edge triggers using cached module hub mode (call after per-monitor setup). */
export function mountEdgeTriggersForMonitor(gdkmonitor: Gdk.Monitor) {
    mountPanelEdgeTriggers(gdkmonitor, cachedModuleHubTrigger)
}
