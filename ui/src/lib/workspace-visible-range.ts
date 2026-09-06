/** Max workspace tab buttons shown in the bar strip at once. */
export const WORKSPACE_MAX_VISIBLE = 10

/**
 * Pick workspace IDs to show in the bar strip.
 *
 * Always includes the active workspace (even when empty). Other tabs are
 * occupied workspaces only. When the total fits in {@link WORKSPACE_MAX_VISIBLE},
 * every occupied workspace is shown. Otherwise a sliding ID window (size =
 * max visible) that includes the active workspace is chosen to maximize how
 * many occupied tabs appear, with ties broken toward lower-numbered workspaces.
 */
export function pickVisibleWorkspaces(
  occupiedIds: Iterable<number>,
  activeId: number | null,
  maxVisible = WORKSPACE_MAX_VISIBLE,
): number[] {
  const occupied = [...occupiedIds]
    .filter((id) => id > 0)
    .sort((a, b) => a - b)
  const safeActive = activeId && activeId > 0 ? activeId : null

  if (occupied.length === 0) {
    return safeActive ? [safeActive] : []
  }

  const activeIsOccupied = safeActive != null && occupied.includes(safeActive)
  const totalTabs = occupied.length + (safeActive && !activeIsOccupied ? 1 : 0)
  if (totalTabs <= maxVisible) {
    const ids = new Set(occupied)
    if (safeActive) ids.add(safeActive)
    return [...ids].sort((a, b) => a - b)
  }

  const windowSize = maxVisible
  const maxId = Math.max(occupied[occupied.length - 1]!, safeActive ?? 1)
  const minStart = 1
  const maxStart = Math.max(1, maxId - windowSize + 1)

  const startMin = safeActive ? Math.max(1, safeActive - windowSize + 1) : minStart
  const startMax = safeActive ? safeActive : maxStart

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
  const ids = new Set(occupied.filter((id) => id >= bestStart && id <= end))
  if (safeActive) ids.add(safeActive)
  return [...ids].sort((a, b) => a - b)
}

/** @deprecated Use {@link pickVisibleWorkspaces}. */
export const pickVisibleOccupiedWorkspaces = pickVisibleWorkspaces
