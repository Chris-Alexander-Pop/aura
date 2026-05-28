import type { ReactNode } from "react"
import { cn } from "@/lib/utils"

export interface PaneHeaderProps {
  /** Material Symbols Rounded ligature name */
  icon?: string
  title: string
  description?: ReactNode
  className?: string
  iconClassName?: string
  titleClassName?: string
}

export function PaneHeader({
  icon,
  title,
  description,
  className,
  iconClassName,
  titleClassName,
}: PaneHeaderProps) {
  return (
    <header className={cn(className)}>
      <div className="flex items-center gap-3">
        {icon != null && icon !== "" && (
          <span className={cn("icon text-mauve text-2xl shrink-0", iconClassName)}>{icon}</span>
        )}
        <h2 className={cn("text-xl font-semibold text-text", titleClassName)}>{title}</h2>
      </div>
      {description != null && (
        <p className="text-xs text-subtext1 mt-1 max-w-prose">{description}</p>
      )}
    </header>
  )
}
