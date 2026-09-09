import grokBotIcon from "@/assets/grok-bot.png"
import { iconFromSpecialWorkspace, specialWorkspaceLabel } from "@/lib/special-workspace"
import { cn } from "@/lib/utils"

type Props = {
  name: string
  className?: string
}

/** Overlay glyph: Grok Bot uses the app icon, everything else is a Material ligature. */
export default function SpecialWorkspaceGlyph({ name, className }: Props) {
  const label = specialWorkspaceLabel(name).toLowerCase()
  if (label === "grok") {
    return (
      <img
        src={grokBotIcon}
        alt=""
        className={cn("h-full w-full object-cover", className)}
      />
    )
  }
  return <span className={cn("icon text-[15px] leading-none", className)}>{iconFromSpecialWorkspace(name)}</span>
}

export function isGrokSpecialWorkspace(name: string): boolean {
  return specialWorkspaceLabel(name).toLowerCase() === "grok"
}
