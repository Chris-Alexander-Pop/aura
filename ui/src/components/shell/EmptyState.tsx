import type { ReactNode } from "react"
import { motion } from "framer-motion"
import { cn } from "@/lib/utils"

const enter = { opacity: 0, y: 8 }
const visible = { opacity: 1, y: 0 }

export type EmptyStateVariant = "plain" | "card"

export interface EmptyStateProps {
  message: ReactNode
  /** Material Symbols Rounded ligature name */
  icon?: string
  title?: string
  variant?: EmptyStateVariant
  className?: string
  iconClassName?: string
  titleClassName?: string
  messageClassName?: string
  animated?: boolean
}

export function EmptyState({
  message,
  icon,
  title,
  variant = "card",
  className,
  iconClassName,
  titleClassName,
  messageClassName,
  animated = false,
}: EmptyStateProps) {
  const showIcon = icon != null && icon !== ""

  const body = (
    <>
      {showIcon && (
        <span
          className={cn(
            "icon shrink-0",
            variant === "card" ? "text-3xl text-subtext1" : "text-xl text-subtext1",
            iconClassName
          )}
        >
          {icon}
        </span>
      )}
      {title != null && title !== "" && (
        <p className={cn("text-sm font-medium text-text", titleClassName)}>{title}</p>
      )}
      <div
        className={cn(
          "text-subtext0",
          variant === "card" ? "text-sm max-w-md" : "text-sm max-w-prose",
          messageClassName
        )}
      >
        {message}
      </div>
    </>
  )

  if (variant === "plain") {
    const plain = cn("flex flex-col gap-2", className)
    if (animated) {
      return (
        <motion.div
          layout
          initial={enter}
          animate={visible}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.2 }}
          className={plain}
        >
          {body}
        </motion.div>
      )
    }
    return <div className={plain}>{body}</div>
  }

  const cardRoot = cn(
    "glass-card flex flex-col items-center justify-center text-center gap-3 p-8",
    className
  )

  if (animated) {
    return (
      <motion.div
        layout
        initial={enter}
        animate={visible}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.2 }}
        className={cardRoot}
      >
        {body}
      </motion.div>
    )
  }

  return <div className={cardRoot}>{body}</div>
}
