import { batteryLevelColorClass } from "@/lib/battery-icon"
import { cn } from "@/lib/utils"

type Props = {
  percent: number
  charging?: boolean
  size?: number
  className?: string
}

const BODY = { x: 3, y: 7, w: 16, h: 10, rx: 2 }
const PAD = 1.75
const INNER_W = BODY.w - PAD * 2
const INNER_H = BODY.h - PAD * 2

/** Battery outline with 1%-granular inner fill; red only when low (see batteryLevelColorClass). */
export default function BatteryLevelIcon({ percent, charging, size = 16, className }: Props) {
  const p = Math.max(0, Math.min(100, percent))
  const fillW = p <= 0 ? 0 : p >= 100 ? INNER_W : (p / 100) * INNER_W

  return (
    <svg
      viewBox="0 0 24 24"
      width={size}
      height={size}
      aria-hidden
      className={cn("block shrink-0", batteryLevelColorClass(p, charging), className)}
    >
      {/* Terminal nub */}
      <rect x="19.25" y="10" width="1.75" height="4" rx="0.4" fill="currentColor" />
      {/* Shell */}
      <rect
        x={BODY.x}
        y={BODY.y}
        width={BODY.w}
        height={BODY.h}
        rx={BODY.rx}
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
      />
      {/* Charge level */}
      {fillW > 0 ? (
        <rect
          x={BODY.x + PAD}
          y={BODY.y + PAD}
          width={fillW}
          height={INNER_H}
          rx="0.75"
          fill="currentColor"
          opacity={charging ? 0.45 : 1}
        />
      ) : null}
      {/* Charging bolt */}
      {charging ? (
        <path
          fill="currentColor"
          d="M12.25 8.5 10 12.75h2l-.35 2.75L14.5 11.5H12.5l-.25-3z"
        />
      ) : null}
    </svg>
  )
}
