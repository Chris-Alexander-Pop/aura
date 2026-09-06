import Gdk from "gi://Gdk?version=4.0"

export type MonitorSize = { width: number; height: number }

export type MonitorGeometry = { x: number; y: number; width: number; height: number }

/** Sanitize Hyprland/GDK connector names for window ids (`eDP-1` → `eDP-1`). */
export function sanitizeMonitorTag(raw: string): string {
    return raw.replace(/[^a-zA-Z0-9_-]/g, "-")
}

/** Logical pixel geometry of a Gdk monitor (fallback 0,0 @ 1920×1080). */
export function gdkMonitorGeometry(monitor: Gdk.Monitor): MonitorGeometry {
    try {
        const m = monitor as Gdk.Monitor & {
            get_geometry?: () => MonitorGeometry
            geometry?: MonitorGeometry
        }
        const g = m.get_geometry?.() ?? m.geometry
        return {
            x: g?.x ?? 0,
            y: g?.y ?? 0,
            width: g?.width ?? 1920,
            height: g?.height ?? 1080,
        }
    } catch {
        return { x: 0, y: 0, width: 1920, height: 1080 }
    }
}

/** DRM connector name when available (matches Hyprland monitor `name` on most setups). */
export function gdkMonitorConnector(monitor: Gdk.Monitor): string | null {
    try {
        const connector = (monitor as Gdk.Monitor & { connector?: string }).connector
        if (connector && connector.length > 0) {
            return connector
        }
    } catch {
        /* older Gdk */
    }
    return null
}

/**
 * Stable id for per-monitor shell windows. Prefer DRM connector (unique per port);
 * fall back to geometry so replugged outputs still get distinct tags when needed.
 */
export function monitorTag(monitor: Gdk.Monitor): string {
    const connector = gdkMonitorConnector(monitor)
    if (connector) {
        return sanitizeMonitorTag(connector)
    }
    const g = gdkMonitorGeometry(monitor)
    return sanitizeMonitorTag(`${g.x}-${g.y}-${g.width}x${g.height}`)
}

/** Logical pixel size of a Gdk monitor (fallback 1920×1080). */
export function monitorSize(monitor: Gdk.Monitor): MonitorSize {
    try {
        const m = monitor as Gdk.Monitor & {
            get_geometry?: () => { width: number; height: number }
            geometry?: { width: number; height: number }
        }
        const g = m.get_geometry?.() ?? m.geometry
        return {
            width: g?.width ?? 1920,
            height: g?.height ?? 1080,
        }
    } catch {
        return { width: 1920, height: 1080 }
    }
}

/** Must match `BAR_STRIP_WIDTH_PX` in `ui/src/components/bar/useBarLayoutStore.ts`. */
export const BAR_STRIP_WIDTH_PX = 44

/** Centered top module hub card width and left margin (~½ monitor). */
export function moduleHubLayout(monitor: Gdk.Monitor): { cardWidth: number; marginLeft: number } {
    const { width } = monitorSize(monitor)
    const cardWidth = Math.max(480, Math.min(Math.floor(width * 0.5), 900))
    const marginLeft = Math.floor((width - cardWidth) / 2)
    return { cardWidth, marginLeft }
}

/** Left-edge vertical module hub beside the bar strip. */
export function moduleHubLeftEdgeLayout(monitor: Gdk.Monitor): {
    panelWidth: number
    panelHeight: number
    marginLeft: number
    marginTop: number
    marginBottom: number
} {
    const { width, height } = monitorSize(monitor)
    const panelWidth = Math.min(400, Math.max(320, Math.floor(width * 0.22)))
    return {
        panelWidth,
        panelHeight: height,
        marginLeft: BAR_STRIP_WIDTH_PX,
        marginTop: 0,
        marginBottom: 0,
    }
}

/** @deprecated Use moduleHubLayout */
export function topThirdLayout(monitor: Gdk.Monitor): { cardWidth: number; marginLeft: number } {
    return moduleHubLayout(monitor)
}

/** Vertical margins to center a panel of `panelHeight` on the monitor. */
export function verticalCenterMargins(monitor: Gdk.Monitor, panelHeight: number): {
    marginTop: number
    marginBottom: number
} {
    const { height } = monitorSize(monitor)
    const marginTop = Math.max(0, Math.floor((height - panelHeight) / 2))
    return { marginTop, marginBottom: marginTop }
}

/** Pin a fixed-size card above the bottom-right corner (TOP|RIGHT anchor). */
export function bottomRightCardLayout(
    monitor: Gdk.Monitor,
    panelHeight: number,
    inset = 8
): { marginTop: number; marginRight: number } {
    const { height } = monitorSize(monitor)
    return {
        marginTop: Math.max(0, height - panelHeight - inset),
        marginRight: inset,
    }
}
