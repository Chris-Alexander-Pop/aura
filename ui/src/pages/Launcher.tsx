import { useEffect, useMemo, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import api, { type LauncherAppView } from "@/lib/api"
import { cn } from "@/lib/utils"

export default function Launcher() {
  const [query, setQuery] = useState("")
  const [selected, setSelected] = useState(0)

  const { data: results, isFetching } = useQuery({
    queryKey: ["launcher-query", query],
    queryFn: async () => {
      const q = query.trim()
      if (!q) return api.launcherRecent()
      try {
        return await api.launcherVicinaeQuery(q)
      } catch {
        return api.launcherQuery(q)
      }
    },
    enabled: true,
    staleTime: 5_000,
  })

  const apps = useMemo(() => (results ?? []) as LauncherAppView[], [results])

  useEffect(() => {
    setSelected(0)
  }, [query, apps.length])

  const run = async (app: LauncherAppView) => {
    await api.launcherRun(app.id)
    try {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      ;(window as any).close?.()
    } catch { /* WebKit host */ }
  }

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault()
      setSelected((i) => Math.min(i + 1, Math.max(0, apps.length - 1)))
    } else if (e.key === "ArrowUp") {
      e.preventDefault()
      setSelected((i) => Math.max(i - 1, 0))
    } else if (e.key === "Enter" && apps[selected]) {
      e.preventDefault()
      void run(apps[selected])
    } else if (e.key === "Escape") {
      window.close?.()
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col bg-mantle/95 p-4 text-text backdrop-blur-2xl">
      <input
        autoFocus
        className="w-full rounded-xl border border-surface0/80 bg-base/90 px-4 py-3 text-base outline-none focus:ring-2 focus:ring-mauve/40"
        placeholder="Search apps and commands…"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onKeyDown={onKeyDown}
      />
      <ul className="mt-3 flex flex-1 flex-col gap-1 overflow-y-auto">
        {isFetching && apps.length === 0 ? (
          <li className="px-3 py-2 text-sm text-subtext0">Searching…</li>
        ) : null}
        {apps.map((app, i) => (
          <li key={app.id}>
            <button
              type="button"
              className={cn(
                "flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm transition-colors",
                i === selected ? "bg-mauve/25 text-text" : "hover:bg-surface0/60 text-subtext1"
              )}
              onMouseEnter={() => setSelected(i)}
              onClick={() => void run(app)}
            >
              <span className="icon text-xl text-mauve">apps</span>
              <span className="min-w-0 flex-1 truncate font-medium">{app.name}</span>
            </button>
          </li>
        ))}
        {!isFetching && apps.length === 0 ? (
          <li className="px-3 py-6 text-center text-sm text-subtext0">No matches</li>
        ) : null}
      </ul>
    </div>
  )
}
