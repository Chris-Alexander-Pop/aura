import { motion } from "framer-motion"
import { cn } from "@/lib/utils"

export function HubQuickToggle({
  icon,
  label,
  active,
  onClick,
  disabled,
  title,
}: {
  icon: string
  label: string
  active?: boolean
  onClick?: () => void
  disabled?: boolean
  title?: string
}) {
  return (
    <motion.button
      type="button"
      whileHover={disabled ? undefined : { scale: 1.04 }}
      whileTap={disabled ? undefined : { scale: 0.95 }}
      onClick={disabled ? undefined : onClick}
      title={title}
      disabled={disabled}
      className={cn(
        "toggle-chip flex min-w-0 flex-1 flex-col gap-0.5 px-2 py-1.5 text-center",
        active && !disabled && "active",
        disabled && "pointer-events-none cursor-default opacity-55"
      )}
    >
      <span className="icon text-lg">{icon}</span>
      <span className="line-clamp-2 text-[10px] leading-tight">{label}</span>
    </motion.button>
  )
}
