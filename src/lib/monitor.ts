import Gdk from "gi://Gdk?version=4.0"

export type MonitorSize = { width: number; height: number }

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

/** Centered top-third module hub card width and left margin. */
export function moduleHubLayout(monitor: Gdk.Monitor): { cardWidth: number; marginLeft: number } {
    const { width } = monitorSize(monitor)
    const cardWidth = Math.max(320, Math.floor(width / 3))
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
