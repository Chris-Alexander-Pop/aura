import type { HyprMonitor, HyprWorkspaceRef } from "@/lib/api-types"

/** Hyprland special/scratch workspace visible on a monitor (`specialWorkspace` in hyprctl). */
export function isSpecialWorkspaceOpen(ref: HyprWorkspaceRef | null | undefined): boolean {
  if (!ref) return false
  if (ref.id === 0) return false
  const name = ref.name ?? ""
  if (name.length === 0) return false
  return name.startsWith("special:") || ref.id < 0
}

export type SpecialWorkspaceCover = {
  monitorName: string
  workspaceId: number
  special: HyprWorkspaceRef
}

/** Every monitor output with an open special workspace and the regular ws it covers. */
export function specialWorkspaceCovers(monitors: HyprMonitor[]): SpecialWorkspaceCover[] {
  const out: SpecialWorkspaceCover[] = []
  for (const monitor of monitors) {
    const special = monitor.special_workspace
    if (!isSpecialWorkspaceOpen(special)) continue
    const workspaceId = monitor.active_workspace?.id
    if (typeof workspaceId !== "number" || workspaceId <= 0) continue
    out.push({
      monitorName: monitor.name,
      workspaceId,
      special: special!,
    })
  }
  return out
}

/** Map covered regular workspace id → special overlay (shown on every bar strip). */
export function specialCoverByWorkspaceId(
  covers: SpecialWorkspaceCover[],
): Map<number, SpecialWorkspaceCover> {
  const map = new Map<number, SpecialWorkspaceCover>()
  for (const cover of covers) {
    map.set(cover.workspaceId, cover)
  }
  return map
}

/** Display label — `special:communication` → `communication`. */
export function specialWorkspaceLabel(name: string): string {
  return name.startsWith("special:") ? name.slice("special:".length) : name
}

/** Argument for `hyprctl dispatch togglespecialworkspace …`. */
export function specialWorkspaceToggleArg(name: string): string {
  return specialWorkspaceLabel(name)
}

/** Material icon for known Aura/Hypr special workspace names. */
export function iconFromSpecialWorkspace(name: string): string {
  const label = specialWorkspaceLabel(name).toLowerCase()
  switch (label) {
    case "communication":
      return "chat"
    case "music":
      return "music_note"
    case "todo":
      return "checklist"
    case "sysmon":
      return "monitoring"
    case "special":
      return "layers"
    default:
      return "layers"
  }
}
