import { useEffect, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import api, { type LauncherAppView } from "@/lib/api"
import { cn } from "@/lib/utils"

function hideLauncher() {
  void api.auraToggleWindow("launcher")
}

async function searchLauncher(query: string): Promise<LauncherAppView[]> {
  const q = query.trim()
  if (!q) {
    const recent = await api.launcherRecent()
    return recent.items
  }
  const [desktop, vicinae] = await Promise.all([
    api.launcherQuery(q).catch(() => ({ results: [] as LauncherAppView[] })),
    api.launcherVicinaeQuery(q).catch(() => ({ results: [] as LauncherAppView[] })),
  ])
  const seen = new Set<string>()
  const merged: LauncherAppView[] = []
  for (const app of [...vicinae.results, ...desktop.results]) {
    if (seen.has(app.id)) continue
    seen.add(app.id)
    merged.push(app)
  }
  return merged
}

export default function Launcher() {
  const [query, setQuery] = useState("")
  const [selected, setSelected] = useState(0)
  const [runError, setRunError] = useState<string | null>(null)

  const { data: apps = [], isFetching } = useQuery({
    queryKey: ["launcher-query", query],
    queryFn: () => searchLauncher(query),
    staleTime: 5_000,
  })

  useEffect(() => {
    setSelected(0)
  }, [query, apps.length])

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") hideLauncher()
    }
    window.addEventListener("keydown", onKeyDown)
    return () => window.removeEventListener("keydown", onKeyDown)
  }, [])

  const run = async (app: LauncherAppView) => {
    setRunError(null)
    try {
      await api.launcherRun(app.id)
      hideLauncher()
    } catch (err) {
      setRunError(err instanceof Error ? err.message : "Failed to launch")
    }
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
      e.preventDefault()
      hideLauncher()
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
      {runError ? (
        <p className="mt-2 rounded-lg bg-red/15 px-3 py-2 text-sm text-red" role="alert">
          {runError}
        </p>
      ) : null}
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
              <span className="icon text-xl text-mauve">{app.icon ?? "apps"}</span>
              <span className="min-w-0 flex-1 truncate font-medium">{app.name}</span>
              {app.comment ? (
                <span className="truncate text-xs text-subtext0">{app.comment}</span>
              ) : null}
            </button>
          </li>
        ))}
        {!isFetching && apps.length === 0 ? (
          <li className="px-3 py-6 text-center text-sm text-subtext0">
            {query.trim() ? "No matches" : "No recent apps"}
          </li>
        ) : null}
      </ul>
    </div>
  )
}
