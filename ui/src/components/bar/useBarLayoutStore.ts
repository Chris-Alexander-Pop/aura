/** Fixed vertical sections on the always-on sidebar strip. */
export const BAR_SECTION_IDS = [
  "launcher",
  "tray",
  "workspaces",
  "runningApps",
  "media",
  "connectivity",
  "calendar",
  "clock",
  "power",
] as const

export type BarSectionId = (typeof BAR_SECTION_IDS)[number]

/** Sections hidden from the strip (legacy ids may still exist in saved settings). */
export const HIDDEN_BAR_SECTIONS = new Set<BarSectionId>(["launcher", "tray"])

/** Pinned above the vertical fill (workspaces + focused window). */
export const TOP_BAR_SECTIONS = new Set<BarSectionId>(["workspaces", "runningApps"])

/** Caelestia-like fixed strip: no ad hoc reorder or expandable utility drawer. */
export const DEFAULT_BAR_SECTION_ORDER: BarSectionId[] = [
  "workspaces",
  "runningApps",
  "media",
  "connectivity",
  "calendar",
  "clock",
  "power",
]
