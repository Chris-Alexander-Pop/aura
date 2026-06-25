import { forwardRef, type ButtonHTMLAttributes } from "react"
import { cn } from "@/lib/utils"

type Props = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> & {
  icon: string
  active?: boolean
  size?: "sm" | "md"
}

const sizeClass = {
  sm: { btn: "h-7 w-7", icon: "text-[16px]" },
  md: { btn: "h-8 w-8", icon: "text-[17px]" },
} as const

/** Compact circular icon control for the vertical bar strip. */
const BarIconButton = forwardRef<HTMLButtonElement, Props>(function BarIconButton(
  { icon, active, size = "sm", className, title, ...props },
  ref
) {
  const s = sizeClass[size]
  return (
    <button
      ref={ref}
      type="button"
      title={title}
      className={cn(
        "flex shrink-0 items-center justify-center rounded-full transition-colors",
        s.btn,
        active
          ? "bg-surface1/80 text-text"
          : "text-subtext1 hover:bg-surface1/60 hover:text-text",
        className,
      )}
      {...props}
    >
      <span className={cn("icon block leading-none", s.icon)}>{icon}</span>
    </button>
  )
})

export default BarIconButton
