import { cn } from "@/lib/utils"

export function FlyoutLoading({ label = "Loading…" }: { label?: string }) {
  return (
    <div
      className="flex flex-col items-center justify-center gap-2 rounded-xl border border-surface1/50 bg-surface0/40 px-4 py-8 text-center"
      role="status"
      aria-live="polite"
      aria-busy="true"
    >
      <span className="icon animate-spin text-3xl text-teal">progress_activity</span>
      <p className="text-[11px] text-subtext0">{label}</p>
    </div>
  )
}

export function FlyoutEmpty({
  icon,
  title,
  detail,
  className,
}: {
  icon: string
  title: string
  detail?: string
  className?: string
}) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center gap-2 rounded-xl border border-surface1/50 bg-surface0/40 px-4 py-6 text-center",
        className,
      )}
    >
      <span className="icon text-3xl text-overlay0">{icon}</span>
      <p className="text-[12px] font-medium text-subtext1">{title}</p>
      {detail ? <p className="mx-auto max-w-[16rem] text-[10px] leading-snug text-subtext0">{detail}</p> : null}
    </div>
  )
}
