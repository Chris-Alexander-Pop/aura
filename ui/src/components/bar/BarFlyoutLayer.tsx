import type { ReactNode } from "react"
import type { StatusFlyoutId } from "./useFlyoutHover"

export const BAR_FLYOUT_PANEL_W = 320

type Props = {
  active: StatusFlyoutId | null
  anchorCenterY: number | null
  onEnterPanel: () => void
  onLeavePanel: () => void
  children: (id: StatusFlyoutId) => ReactNode
}

export default function BarFlyoutLayer({ active, anchorCenterY, onEnterPanel, onLeavePanel, children }: Props) {
  if (!active || anchorCenterY == null) return null

  return (
    <div
      className="pointer-events-none absolute inset-y-0 left-14 z-30 flex justify-start"
      style={{ width: BAR_FLYOUT_PANEL_W }}
      aria-hidden
    >
      <div
        className="pointer-events-auto absolute max-h-[min(520px,calc(100vh-48px))] overflow-hidden rounded-r-2xl border border-surface0/90 border-l-transparent bg-mantle/90 text-text shadow-xl backdrop-blur-xl"
        style={{
          top: anchorCenterY,
          transform: "translateY(-50%)",
          width: BAR_FLYOUT_PANEL_W,
        }}
        onMouseEnter={onEnterPanel}
        onMouseLeave={onLeavePanel}
      >
        {children(active)}
      </div>
    </div>
  )
}
