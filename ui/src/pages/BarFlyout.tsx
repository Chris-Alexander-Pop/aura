/**
 * Flyout overlay page (#/bar-flyout) — loaded once in a separate transparent
 * WebKit window positioned immediately to the right of the sidebar strip.
 *
 * Content is driven by custom events dispatched from AGS via evaluate_javascript:
 *   window.dispatchEvent(new CustomEvent('aura-flyout', { detail: { panel, y } }))
 *
 * Mouse enter/leave posts webkit messages ("enter"/"leave") so AGS can manage
 * the close timer with the same grace period as the strip-hover logic.
 */
import { useEffect, useState } from "react"
import AudioFlyout from "@/components/bar/flyouts/AudioFlyout"
import BrightnessFlyout from "@/components/bar/flyouts/BrightnessFlyout"
import NetworkFlyout from "@/components/bar/flyouts/NetworkFlyout"
import BluetoothFlyout from "@/components/bar/flyouts/BluetoothFlyout"
import BatteryFlyout from "@/components/bar/flyouts/BatteryFlyout"
import WindowsFlyout from "@/components/bar/flyouts/WindowsFlyout"
import PowerFlyout from "@/components/bar/flyouts/PowerFlyout"
import { AnimatePresence, motion } from "framer-motion"

const FLYOUT_W = 320

function postHover(hovered: boolean) {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(window as any).webkit?.messageHandlers?.barFlyoutHover?.postMessage?.(hovered ? "enter" : "leave")
  } catch { /* non-WebKit context */ }
}

export default function BarFlyout() {
  const [state, setState] = useState<{ panel: string; y: number } | null>(null)

  useEffect(() => {
    document.documentElement.classList.add("aura-bar-host")
    return () => document.documentElement.classList.remove("aura-bar-host")
  }, [])

  useEffect(() => {
    const handler = (e: Event) => {
      const { panel, y } = (e as CustomEvent<{ panel: string; y: number }>).detail
      setState({ panel, y })
    }
    window.addEventListener("aura-flyout", handler)
    return () => window.removeEventListener("aura-flyout", handler)
  }, [])

  const content = (() => {
    switch (state?.panel) {
      case "network":   return <NetworkFlyout />
      case "bluetooth": return <BluetoothFlyout />
      case "audio":     return <AudioFlyout />
      case "brightness": return <BrightnessFlyout />
      case "battery":   return <BatteryFlyout />
      case "windows":   return <WindowsFlyout />
      case "power":     return <PowerFlyout />
      default:          return null
    }
  })()

  return (
    <div className="aura-bar-root relative h-full w-full pointer-events-none">
      <AnimatePresence mode="wait">
        {content && state ? (
          <motion.div
            key={state.panel}
            className="pointer-events-auto absolute max-h-[min(520px,calc(100vh-48px))] origin-left overflow-hidden rounded-r-2xl border border-surface0/90 border-l-transparent bg-mantle/90 text-text shadow-xl backdrop-blur-xl"
            style={{ top: state.y, width: FLYOUT_W }}
            initial={{ opacity: 0, x: -10, y: "-50%", scale: 0.96 }}
            animate={{ opacity: 1, x: 0, y: "-50%", scale: 1 }}
            exit={{ opacity: 0, x: -8, y: "-50%", scale: 0.97 }}
            transition={{ type: "spring", stiffness: 420, damping: 34, mass: 0.7 }}
            onMouseEnter={() => postHover(true)}
            onMouseLeave={() => postHover(false)}
          >
            {content}
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  )
}
