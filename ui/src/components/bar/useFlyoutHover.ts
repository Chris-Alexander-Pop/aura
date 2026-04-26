import { useCallback, useEffect, useRef, useState } from "react"

export type StatusFlyoutId = "network" | "bluetooth" | "battery" | "windows" | "power"

const CLOSE_MS = 220

/** Caelestia-like hover: open on icon enter, grace period when moving into panel */
export function useFlyoutHover() {
  const [active, setActive] = useState<StatusFlyoutId | null>(null)
  const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)

  const cancelClose = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current)
      timerRef.current = undefined
    }
  }, [])

  const scheduleClose = useCallback(() => {
    cancelClose()
    timerRef.current = setTimeout(() => setActive(null), CLOSE_MS)
  }, [cancelClose])

  const open = useCallback(
    (id: StatusFlyoutId) => {
      cancelClose()
      setActive(id)
    },
    [cancelClose]
  )

  const onEnterFlyout = useCallback(() => {
    cancelClose()
  }, [cancelClose])

  useEffect(() => () => cancelClose(), [cancelClose])

  return { active, open, scheduleClose, onEnterFlyout, closeNow: () => setActive(null) }
}
