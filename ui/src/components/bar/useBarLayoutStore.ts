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

/** Caelestia-like fixed strip: no ad hoc reorder or expandable utility drawer. */
export const DEFAULT_BAR_SECTION_ORDER: BarSectionId[] = [
  "launcher",
  "tray",
  "workspaces",
  "runningApps",
  "media",
  "connectivity",
  "calendar",
  "clock",
  "power",
]
