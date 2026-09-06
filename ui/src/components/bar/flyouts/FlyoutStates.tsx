import { cn } from "@/lib/utils"

export function FlyoutLoading({ label = "Loading…" }: { label?: string }) {
  return (
    <div
      className="flex items-center gap-2 py-1 text-subtext0"
      role="status"
      aria-live="polite"
      aria-busy="true"
    >
      <span className="icon animate-spin text-base text-teal">progress_activity</span>
      <p className="text-[10px]">{label}</p>
    </div>
  )
}

export function FlyoutEmpty({
  icon,
  title,
  detail,
  className,
}: {
  icon?: string
  title: string
  detail?: string
  className?: string
}) {
  return (
    <div className={cn("flex items-start gap-1.5 py-1 text-subtext0", className)}>
      {icon ? <span className="icon shrink-0 text-sm text-overlay0">{icon}</span> : null}
      <div className="min-w-0">
        <p className="text-[10px] font-medium text-subtext1">{title}</p>
        {detail ? <p className="text-[10px] leading-snug">{detail}</p> : null}
      </div>
    </div>
  )
}
