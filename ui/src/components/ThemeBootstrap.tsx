import { useEffect } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import { applyThemeToDocument } from "@/lib/applyTheme"
import { connectWs, useWsStore } from "@/lib/ws"

/** Load Settings.theme on boot and when settings change. */
export default function ThemeBootstrap() {
  const qc = useQueryClient()
  const { data } = useQuery({
    queryKey: ["aura-settings"],
    queryFn: api.getAuraSettings,
    staleTime: 60_000,
  })

  useEffect(() => {
    applyThemeToDocument(data?.settings.theme)
  }, [data?.settings.theme])

  useEffect(() => {
    connectWs()
    const off = useWsStore.getState().on("Settings.Changed", () => {
      void qc.invalidateQueries({ queryKey: ["aura-settings"] })
    })
    return off
  }, [qc])

  return null
}
