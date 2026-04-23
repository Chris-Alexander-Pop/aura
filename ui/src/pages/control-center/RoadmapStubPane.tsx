import { motion } from "framer-motion"
import { getNavItem, type PaneId } from "./navigation"
import { ROADMAP_STUB_BLURBS } from "./stubContent"

type StubId = keyof typeof ROADMAP_STUB_BLURBS

export function RoadmapStubPane({ paneId }: { paneId: StubId }) {
  const { icon, label } = getNavItem(paneId)
  const blurb = ROADMAP_STUB_BLURBS[paneId]

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-6 p-6 h-full overflow-y-auto"
    >
      <div>
        <div className="flex items-center gap-3 mb-2">
          <span className="icon text-mauve text-2xl">{icon}</span>
          <h2 className="text-xl font-semibold text-text">{label}</h2>
        </div>
        <p className="text-sm text-subtext0 leading-relaxed max-w-prose">{blurb}</p>
        <p className="text-xs text-subtext1 mt-3">Implementation stub — `docs/roadmap/control-panel.md`</p>
      </div>
      {[...Array(4)].map((_, i) => (
        <div key={i} className="skeleton h-16 rounded-xl" style={{ opacity: 1 - i * 0.15 }} />
      ))}
    </motion.div>
  )
}

export function isStubPaneId(id: PaneId): id is StubId {
  return id in ROADMAP_STUB_BLURBS
}
