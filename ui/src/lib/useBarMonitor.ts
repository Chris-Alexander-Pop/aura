import { useEffect, useMemo, useState } from "react"
import { useSearchParams } from "react-router-dom"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { parseHyprMonitors } from "@/lib/api-types"
import { hyprlandQueryDefaults } from "@/lib/useHyprlandSync"

declare global {
  interface Window {
    __AURA_BAR_MONITOR__?: string
  }
}

function resolveHyprMonitorName(
  monitors: ReturnType<typeof parseHyprMonitors>,
  ...candidates: Array<string | null | undefined>
): string | null {
  for (const candidate of candidates) {
    if (!candidate) continue
    const exact = monitors.find((m) => m.name === candidate)
    if (exact) return exact.name
    const fold = monitors.find((m) => m.name.toLowerCase() === candidate.toLowerCase())
    if (fold) return fold.name
  }
  return null
}

/** Hypr monitor name for this bar WebView (injected by AGS, URL param, or focused output). */
export function useBarMonitorName(): string | null {
  const [params] = useSearchParams()
  const fromUrl = params.get("monitor")
  const [injected, setInjected] = useState<string | null>(
    () => (typeof window !== "undefined" ? window.__AURA_BAR_MONITOR__ ?? null : null),
  )

  useEffect(() => {
    if (window.__AURA_BAR_MONITOR__) {
      setInjected(window.__AURA_BAR_MONITOR__ ?? null)
    }
    const onMonitor = (e: Event) => {
      const name = (e as CustomEvent<{ name?: string }>).detail?.name
      if (typeof name === "string" && name.length > 0) setInjected(name)
    }
    window.addEventListener("aura-bar-monitor", onMonitor)
    return () => window.removeEventListener("aura-bar-monitor", onMonitor)
  }, [])

  const { data: monitorsRaw } = useQuery({
    queryKey: ["hypr-monitors"],
    queryFn: api.hyprlandGetMonitors,
    ...hyprlandQueryDefaults,
  })

  return useMemo(() => {
    const monitors = parseHyprMonitors(monitorsRaw)
    const resolved = resolveHyprMonitorName(monitors, injected, fromUrl)
    if (resolved) return resolved
    return monitors.find((m) => m.focused)?.name ?? monitors[0]?.name ?? injected ?? fromUrl
  }, [injected, fromUrl, monitorsRaw])
}
