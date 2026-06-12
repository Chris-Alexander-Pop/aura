import type { SidebarTileDataView } from "@/lib/api-types"
import type { PaneId } from "@/pages/control-center/navigation"

/** Module ids with `Sidebar.GetTileData` support (matches sidecar `dashboard.rs`). */
export const DROPDOWN_TILE_IDS = [
  "network",
  "audio",
  "bluetooth",
  "battery",
  "calendar",
  "notifications",
  "productivity",
] as const

export type DropdownTileId = (typeof DROPDOWN_TILE_IDS)[number]

/** Alias for sidebar page — same tile set as dropdown modules. */
export const SIDEBAR_TILE_IDS = DROPDOWN_TILE_IDS
export type SidebarTileId = DropdownTileId

export const DROPDOWN_TILE_LABELS: Record<DropdownTileId, string> = {
  network: "Network",
  audio: "Audio",
  bluetooth: "Bluetooth",
  battery: "Battery",
  calendar: "Calendar",
  notifications: "Notifications",
  productivity: "Focus",
}

export const SIDEBAR_TILE_LABELS = DROPDOWN_TILE_LABELS

const TILE_PANE: Partial<Record<DropdownTileId, PaneId>> = {
  network: "network",
  audio: "audio",
  bluetooth: "bluetooth",
  battery: "performance",
  notifications: "notifications",
  productivity: "productivity",
}

export const SIDEBAR_TILE_PANE = TILE_PANE

export function filterDropdownModules(ids: string[]): DropdownTileId[] {
  const allowed = new Set<string>(DROPDOWN_TILE_IDS)
  return ids.filter((id): id is DropdownTileId => allowed.has(id))
}

/** Control center pane for a tile, or `null` when the tile opens a dedicated window. */
export function dropdownTilePane(tileId: DropdownTileId): PaneId | null {
  if (tileId === "calendar") return null
  return TILE_PANE[tileId] ?? null
}

export function isDropdownTileId(id: string): id is DropdownTileId {
  return (DROPDOWN_TILE_IDS as readonly string[]).includes(id)
}

export const isSidebarTileId = isDropdownTileId

export function sidebarTileSummary(
  tileId: string,
  data: SidebarTileDataView | undefined,
  opts?: { loading?: boolean; error?: boolean }
): string {
  if (opts?.loading) return "…"
  if (opts?.error || !data) return "Unavailable"

  const d = data.data
  switch (tileId) {
    case "network":
      return (
        (d.active_connection as string) ||
        ((d.wifi_enabled as boolean) ? "Wi‑Fi on" : "Wi‑Fi off")
      )
    case "audio": {
      const sink = (d.default_sink as { name?: string; volume?: number }) ?? {}
      return sink.name
        ? `${sink.name} · ${Math.round((sink.volume ?? 0) * 100)}%`
        : "No output"
    }
    case "bluetooth":
      return (d.powered as boolean) ? `${d.connected_count ?? 0} linked` : "Off"
    case "battery": {
      const b = (d.battery as { percent?: number; charging?: boolean }) ?? {}
      const profile = d.power_profile as string | undefined
      const pct =
        typeof b.percent === "number"
          ? `${b.percent}%${b.charging ? " ⚡" : ""}`
          : "—"
      return profile ? `${pct} · ${profile}` : pct
    }
    case "calendar": {
      const events = (d.events as unknown[]) ?? []
      return events.length ? `${events.length} upcoming` : "No events"
    }
    case "notifications": {
      const dnd = (d.dnd as { enabled?: boolean }) ?? {}
      return dnd.enabled ? "DND on" : "Inbox"
    }
    case "productivity":
      return (d.focus_mode_enabled as boolean) ? "Focus on" : "Focus off"
    default:
      return "Open panel"
  }
}
