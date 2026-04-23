import { create } from "zustand"
import { persist } from "zustand/middleware"

/** Ordered vertical sections on the React bar — persisted for Phase 3 layout config */
export const BAR_SECTION_IDS = [
  "logo",
  "workspaces",
  "activeWindow",
  "media",
  "connectivity",
  "calendar",
  "launcher",
  "clock",
  "power",
  "more",
  "trayNote",
  "systemStats",
  "windowHub",
  "taskMgr",
] as const

export type BarSectionId = (typeof BAR_SECTION_IDS)[number]

/** Caelestia-like sparse strip: hub/stats/tasks/tray live under “More…” */
export const DEFAULT_BAR_SECTION_ORDER: BarSectionId[] = [
  "logo",
  "workspaces",
  "activeWindow",
  "media",
  "calendar",
  "launcher",
  "connectivity",
  "clock",
  "power",
  "more",
]

function defaultOrder(): BarSectionId[] {
  return [...DEFAULT_BAR_SECTION_ORDER]
}

interface BarLayoutState {
  sectionOrder: BarSectionId[]
  setSectionOrder: (order: BarSectionId[]) => void
  resetOrder: () => void
}

export const useBarLayoutStore = create<BarLayoutState>()(
  persist(
    (set) => ({
      sectionOrder: defaultOrder(),
      setSectionOrder: (sectionOrder) => set({ sectionOrder }),
      resetOrder: () => set({ sectionOrder: defaultOrder() }),
    }),
    { name: "aura-bar-strip-layout-v3" }
  )
)
