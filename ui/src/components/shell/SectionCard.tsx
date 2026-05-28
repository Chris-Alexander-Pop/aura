import type { ReactNode } from "react"
import { motion } from "framer-motion"
import { cn } from "@/lib/utils"

const enter = { opacity: 0, y: 8 }
const visible = { opacity: 1, y: 0 }

export interface SectionCardProps {
  children: ReactNode
  /** Merged onto the glass-card root */
  className?: string
  /** Default inner layout stacks content like Control Center cards */
  contentClassName?: string
  /** Match pane content cards: padded flex column with gap */
  padded?: boolean
  animated?: boolean
}

export function SectionCard({
  children,
  className,
  contentClassName,
  padded = true,
  animated = false,
}: SectionCardProps) {
  const root = cn(
    "glass-card",
    padded && "flex flex-col gap-3 p-4",
    contentClassName,
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
        className={root}
      >
        {children}
      </motion.div>
    )
  }

  return <div className={root}>{children}</div>
}
