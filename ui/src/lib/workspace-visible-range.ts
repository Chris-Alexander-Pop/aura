/** Max workspace tab buttons shown in the bar strip at once. */
export const WORKSPACE_VISIBLE_WINDOW = 5

/**
 * Pick occupied workspace IDs to show in the bar strip.
 *
 * Scans contiguous windows of {@link WORKSPACE_VISIBLE_WINDOW} slots, keeps the
 * window with the most occupied tabs, and breaks ties toward lower-numbered
 * workspaces. When the active workspace has windows, only windows that include
 * it are considered so the current workspace stays reachable.
 */
export function pickVisibleOccupiedWorkspaces(
  occupiedIds: Iterable<number>,
  activeId: number | null,
  windowSize = WORKSPACE_VISIBLE_WINDOW,
): number[] {
  const occupied = [...occupiedIds].sort((a, b) => a - b)
  if (occupied.length === 0) return []

  const occupiedSet = new Set(occupied)
  const safeActive = activeId && activeId > 0 ? activeId : null
  const activeIsOccupied = safeActive != null && occupiedSet.has(safeActive)

  const maxId = Math.max(occupied[occupied.length - 1]!, safeActive ?? 1, windowSize)
  const minStart = 1
  const maxStart = Math.max(1, maxId - windowSize + 1)

  // Empty active workspace: pick the globally best window (don't anchor to it).
  const startMin = activeIsOccupied ? Math.max(1, safeActive! - windowSize + 1) : minStart
  const startMax = activeIsOccupied ? safeActive! : maxStart

  let bestStart = startMin
  let bestCount = -1

  for (let start = startMin; start <= startMax; start++) {
    const end = start + windowSize - 1
    const count = occupied.filter((id) => id >= start && id <= end).length
    if (count > bestCount || (count === bestCount && start < bestStart)) {
      bestCount = count
      bestStart = start
    }
  }

  const end = bestStart + windowSize - 1
  return occupied.filter((id) => id >= bestStart && id <= end)
}
