import { useEffect, useRef, useState, type KeyboardEvent, type PointerEvent, type ReactNode } from "react"
import { cn } from "@/lib/utils"

export function FlyoutShell({
  children,
  className,
}: {
  children: ReactNode
  className?: string
}) {
  return (
    <div className={cn("flex flex-col gap-1.5 px-2.5 py-2 text-text", className)}>
      {children}
    </div>
  )
}

export function FlyoutTitle({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <h2 className={cn("pr-1 text-xs font-medium leading-tight text-subtext1", className)}>{children}</h2>
  )
}

export function FlyoutToggleRow({
  label,
  checked,
  disabled,
  onChange,
}: {
  label: string
  checked: boolean
  disabled?: boolean
  onChange: (checked: boolean) => void
}) {
  return (
    <div className="flex items-center justify-between gap-2 py-0.5">
      <span className="text-[11px] text-text">{label}</span>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        aria-label={label}
        disabled={disabled}
        className={cn(
          "relative inline-flex h-5 w-9 shrink-0 rounded-full border transition-colors duration-200",
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-teal/40 focus-visible:ring-offset-1 focus-visible:ring-offset-base",
          checked ? "border-teal bg-teal" : "border-overlay0/50 bg-surface0/90",
          disabled ? "cursor-not-allowed opacity-40" : "cursor-pointer",
        )}
        onClick={() => onChange(!checked)}
      >
        <span
          aria-hidden
          className={cn(
            "pointer-events-none absolute top-1/2 left-0.5 h-3.5 w-3.5 -translate-y-1/2 rounded-full bg-crust shadow-sm",
            "transition-transform duration-200 ease-out",
            checked && "translate-x-[18px]",
          )}
        />
      </button>
    </div>
  )
}

export function FlyoutMeta({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <p className={cn("text-[10px] leading-snug text-subtext0", className)}>{children}</p>
  )
}

export function FlyoutBanner({
  children,
  tone = "warn",
}: {
  children: ReactNode
  tone?: "warn" | "error"
}) {
  return (
    <p
      className={cn(
        "text-[10px] leading-snug",
        tone === "error" ? "text-red" : "text-yellow",
      )}
      role={tone === "error" ? "alert" : undefined}
    >
      {children}
    </p>
  )
}

export function FlyoutList({
  children,
  className,
}: {
  children: ReactNode
  className?: string
}) {
  return (
    <ul className={cn("flex max-h-44 flex-col gap-0.5 overflow-y-auto pr-0.5", className)}>
      {children}
    </ul>
  )
}

export function FlyoutRow({
  children,
  className,
}: {
  children: ReactNode
  className?: string
}) {
  return (
    <li className={cn("flex min-h-8 items-center gap-1.5 py-0.5", className)}>{children}</li>
  )
}

export function FlyoutRowIcon({
  icon,
  active,
  className,
}: {
  icon: string
  active?: boolean
  className?: string
}) {
  return (
    <span
      className={cn(
        "icon shrink-0 text-[16px] leading-none",
        active ? "text-teal" : "text-subtext0",
        className,
      )}
    >
      {icon}
    </span>
  )
}

export function FlyoutRowLabel({
  children,
  active,
  className,
}: {
  children: ReactNode
  active?: boolean
  className?: string
}) {
  return (
    <span
      className={cn(
        "min-w-0 flex-1 truncate text-[11px] leading-tight",
        active ? "font-medium text-teal" : "text-subtext1",
        className,
      )}
    >
      {children}
    </span>
  )
}

export function FlyoutIconButton({
  icon,
  active,
  disabled,
  loading,
  title,
  onClick,
}: {
  icon: string
  active?: boolean
  disabled?: boolean
  loading?: boolean
  title?: string
  onClick: () => void
}) {
  return (
    <button
      type="button"
      disabled={disabled || loading}
      title={title}
      className={cn(
        "flex h-7 w-7 shrink-0 items-center justify-center rounded-full transition-colors disabled:opacity-40",
        active ? "bg-teal text-crust" : "text-subtext1 hover:bg-surface0/80 hover:text-text",
      )}
      onClick={onClick}
    >
      {loading ? (
        <span className="icon animate-spin text-[14px]">progress_activity</span>
      ) : (
        <span className="icon text-[16px]">{icon}</span>
      )}
    </button>
  )
}

export function FlyoutActionButton({
  children,
  icon,
  disabled,
  loading,
  onClick,
  variant = "default",
}: {
  children: ReactNode
  icon?: string
  disabled?: boolean
  loading?: boolean
  onClick: () => void
  variant?: "default" | "primary"
}) {
  return (
    <button
      type="button"
      disabled={disabled || loading}
      className={cn(
        "flex w-full items-center justify-center gap-1.5 rounded-full py-2 text-[11px] font-medium transition-colors disabled:opacity-40",
        variant === "primary"
          ? "bg-blue/25 text-blue hover:bg-blue/35"
          : "bg-surface0/60 text-subtext1 hover:bg-surface0 hover:text-text",
      )}
      onClick={onClick}
    >
      {icon ? (
        <span className={cn("icon text-base", loading && "animate-spin")}>{loading ? "progress_activity" : icon}</span>
      ) : null}
      {children}
    </button>
  )
}

export function FlyoutExpandLink({
  label,
  onClick,
}: {
  label: string
  onClick: () => void
}) {
  return (
    <button
      type="button"
      className="flex w-full items-center justify-between gap-1 rounded-lg bg-surface0/50 px-2 py-1.5 text-[10px] font-medium text-subtext1 transition-colors hover:bg-surface0 hover:text-text"
      onClick={onClick}
    >
      <span>{label}</span>
      <span className="icon text-sm">chevron_right</span>
    </button>
  )
}

export function FlyoutSectionLabel({ children }: { children: ReactNode }) {
  return (
    <p className="text-[10px] font-semibold uppercase tracking-wide text-subtext1">{children}</p>
  )
}

export function FlyoutRadioRow({
  label,
  checked,
  disabled,
  onSelect,
}: {
  label: string
  checked: boolean
  disabled?: boolean
  onSelect: () => void
}) {
  return (
    <button
      type="button"
      disabled={disabled || checked}
      className={cn(
        "flex h-8 w-full items-center gap-2 rounded-lg px-1.5 text-left text-[11px] transition-colors",
        checked ? "bg-mauve/15 text-text" : "text-subtext1 hover:bg-surface0/70",
        disabled && !checked && "opacity-60",
      )}
      onClick={onSelect}
    >
      <span
        className={cn(
          "flex h-3.5 w-3.5 shrink-0 items-center justify-center rounded-full border",
          checked ? "border-teal bg-teal" : "border-overlay0",
        )}
      >
        {checked ? <span className="h-1.5 w-1.5 rounded-full bg-crust" /> : null}
      </span>
      <span className="min-w-0 flex-1 truncate">{label}</span>
    </button>
  )
}

function clampStep(value: number, min: number, max: number, step: number): number {
  const stepped = Math.round(value / step) * step
  return Math.min(max, Math.max(min, stepped))
}

export function FlyoutSlider({
  value,
  min = 0,
  max = 100,
  step = 1,
  disabled,
  label,
  accent = "teal",
  live = false,
  showThumb = true,
  onLiveChange,
  onChange,
}: {
  value: number
  min?: number
  max?: number
  step?: number
  disabled?: boolean
  label: string
  accent?: "teal" | "amber" | "sapphire"
  live?: boolean
  showThumb?: boolean
  /** Debounced updates while dragging when `live` is set. Falls back to `onChange`. */
  onLiveChange?: (value: number) => void
  onChange: (value: number) => void
}) {
  const trackRef = useRef<HTMLDivElement>(null)
  const [local, setLocal] = useState(() => clampStep(value, min, max, step))
  const draggingRef = useRef(false)
  const liveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    if (!draggingRef.current) {
      setLocal(clampStep(value, min, max, step))
    }
  }, [value, min, max, step])

  useEffect(
    () => () => {
      if (liveTimerRef.current != null) clearTimeout(liveTimerRef.current)
    },
    [],
  )

  const fillPct = max === min ? 0 : ((local - min) / (max - min)) * 100

  const accentFill =
    accent === "amber" ? "bg-amber" : accent === "sapphire" ? "bg-sapphire" : "bg-teal"

  const valueFromClientX = (clientX: number) => {
    const track = trackRef.current
    if (!track) return local
    const { left, width } = track.getBoundingClientRect()
    if (width <= 0) return local
    const ratio = Math.max(0, Math.min(1, (clientX - left) / width))
    return clampStep(min + ratio * (max - min), min, max, step)
  }

  const commit = (v: number) => {
    if (liveTimerRef.current != null) {
      clearTimeout(liveTimerRef.current)
      liveTimerRef.current = null
    }
    onChange(v)
  }

  const scheduleLive = (v: number) => {
    if (!live) return
    const liveHandler = onLiveChange ?? onChange
    if (liveTimerRef.current != null) clearTimeout(liveTimerRef.current)
    liveTimerRef.current = setTimeout(() => {
      liveTimerRef.current = null
      liveHandler(v)
    }, 40)
  }

  const onPointerDown = (e: PointerEvent<HTMLDivElement>) => {
    if (disabled) return
    draggingRef.current = true
    e.currentTarget.setPointerCapture(e.pointerId)
    const v = valueFromClientX(e.clientX)
    setLocal(v)
    scheduleLive(v)
  }

  const onPointerMove = (e: PointerEvent<HTMLDivElement>) => {
    if (!draggingRef.current || disabled) return
    const v = valueFromClientX(e.clientX)
    setLocal(v)
    scheduleLive(v)
  }

  const endDrag = (e: PointerEvent<HTMLDivElement>) => {
    if (!draggingRef.current) return
    draggingRef.current = false
    if (e.currentTarget.hasPointerCapture(e.pointerId)) {
      e.currentTarget.releasePointerCapture(e.pointerId)
    }
    const v = valueFromClientX(e.clientX)
    setLocal(v)
    commit(v)
  }

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (disabled) return
    let next: number | null = null
    if (e.key === "ArrowRight" || e.key === "ArrowUp") next = Math.min(max, local + step)
    else if (e.key === "ArrowLeft" || e.key === "ArrowDown") next = Math.max(min, local - step)
    else if (e.key === "Home") next = min
    else if (e.key === "End") next = max
    if (next == null) return
    e.preventDefault()
    setLocal(next)
    commit(next)
  }

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between gap-2">
        <p className="text-[10px] font-medium text-subtext1">{label}</p>
        <p className="text-[10px] tabular-nums text-subtext0">{local}%</p>
      </div>
      <div
        ref={trackRef}
        role="slider"
        tabIndex={disabled ? -1 : 0}
        aria-valuemin={min}
        aria-valuemax={max}
        aria-valuenow={local}
        aria-label={label}
        className={cn(
          "relative h-2 w-full touch-none rounded-full border border-surface1/40 bg-base/90 outline-none",
          "focus-visible:ring-2 focus-visible:ring-teal/40 focus-visible:ring-offset-1 focus-visible:ring-offset-base",
          disabled ? "cursor-not-allowed opacity-50" : "cursor-pointer",
        )}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={endDrag}
        onPointerCancel={endDrag}
        onKeyDown={onKeyDown}
      >
        <div
          className={cn(
            "pointer-events-none absolute inset-y-0 left-0 rounded-full",
            showThumb ? "opacity-75" : "opacity-100",
            accentFill,
          )}
          style={{ width: `${fillPct}%` }}
        />
        {showThumb ? (
          <div
            className={cn(
              "pointer-events-none absolute top-1/2 h-3 w-3 -translate-y-1/2 rounded-full shadow-sm ring-2 ring-mantle/80",
              accentFill,
            )}
            style={{ left: `calc(${fillPct}% - 6px)` }}
          />
        ) : null}
      </div>
    </div>
  )
}

export function FlyoutProfilePicker({
  value,
  disabled,
  onChange,
}: {
  value: "saver" | "balanced" | "performance"
  disabled?: boolean
  onChange: (profile: "saver" | "balanced" | "performance") => void
}) {
  const segments = [
    { id: "saver" as const, icon: "energy_savings_leaf", title: "Power saver" },
    { id: "balanced" as const, icon: "balance", title: "Balanced" },
    { id: "performance" as const, icon: "rocket_launch", title: "Performance" },
  ]

  const activeIndex = segments.findIndex((s) => s.id === value)

  return (
    <div className="relative flex items-center justify-between rounded-full bg-surface0/80 p-0.5">
      <div
        className="absolute top-0.5 bottom-0.5 rounded-full bg-teal transition-[left,width] duration-200 ease-out"
        style={{
          left: `calc(${activeIndex} * (100% / 3) + 2px)`,
          width: `calc(100% / 3 - 4px)`,
        }}
      />
      {segments.map((seg) => {
        const active = value === seg.id
        return (
          <button
            key={seg.id}
            type="button"
            title={seg.title}
            disabled={disabled}
            className={cn(
              "relative z-10 flex flex-1 items-center justify-center py-1.5 transition-colors disabled:opacity-50",
            )}
            onClick={() => onChange(seg.id)}
          >
            <span
              className={cn(
                "icon text-[18px]",
                active ? "text-crust" : "text-subtext1",
              )}
            >
              {seg.icon}
            </span>
          </button>
        )
      })}
    </div>
  )
}

export function FlyoutActionRow({
  icon,
  label,
  destructive,
  disabled,
  loading,
  onClick,
}: {
  icon: string
  label: string
  destructive?: boolean
  disabled?: boolean
  loading?: boolean
  onClick: () => void
}) {
  return (
    <button
      type="button"
      disabled={disabled || loading}
      className={cn(
        "flex h-9 w-full items-center gap-2 rounded-lg px-2 text-left text-[11px] transition-colors disabled:opacity-50",
        destructive
          ? "text-red hover:bg-red/10"
          : "text-subtext1 hover:bg-surface0/80 hover:text-text",
      )}
      onClick={onClick}
    >
      <span className={cn("icon text-[18px]", loading && "animate-spin")}>
        {loading ? "progress_activity" : icon}
      </span>
      <span className="font-medium">{label}</span>
    </button>
  )
}
