import { useMemo } from "react"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"
import { iconFromHyprClass } from "@/components/bar/hyprWindowIcon"

type HyprWorkspaceRef = { id?: number; name?: string }

type HyprClient = {
  address?: string
  title?: string
  class?: string
  workspace?: HyprWorkspaceRef
  floating?: boolean
}

function parseClients(raw: unknown): HyprClient[] {
  if (!Array.isArray(raw)) return []
  return raw.filter((c): c is HyprClient => c != null && typeof c === "object")
}

function wsKey(c: HyprClient): number {
  const id = c.workspace?.id
  return typeof id === "number" && Number.isFinite(id) ? id : -1
}

export default function WindowsFlyout() {
  const { data: rawClients } = useQuery({ queryKey: ["clients"], queryFn: api.hyprlandGetClients, refetchInterval: 1500 })
  const { data: rawActive } = useQuery({ queryKey: ["hypr-active"], queryFn: api.hyprlandGetActiveWindow, refetchInterval: 1000 })

  const activeAddr =
    rawActive && typeof rawActive === "object" && "address" in rawActive
      ? String((rawActive as { address?: unknown }).address ?? "")
      : ""

  const clients = useMemo(() => parseClients(rawClients).filter((c) => c.address), [rawClients])

  const grouped = useMemo(() => {
    const byWs = new Map<number, HyprClient[]>()
    for (const c of clients) {
      const k = wsKey(c)
      if (!byWs.has(k)) byWs.set(k, [])
      byWs.get(k)!.push(c)
    }
    const sortedKeys = [...byWs.keys()].sort((a, b) => a - b)
    return sortedKeys.map((id) => ({
      wsId: id,
      label: id < 0 ? "?" : String(id),
      items: byWs.get(id)!.sort((a, b) =>
        String(a.title ?? "").localeCompare(String(b.title ?? ""), undefined, { sensitivity: "base" })
      ),
    }))
  }, [clients])

  const activeClient = useMemo(
    () => clients.find((c) => c.address === activeAddr) ?? null,
    [clients, activeAddr]
  )

  const focusAddr = (addr: string) => {
    if (addr) void api.hyprlandDispatch(`focuswindow address:${addr}`)
  }

  const activeTitle = activeClient ? String(activeClient.title ?? "").trim() : ""
  const activeClass = activeClient ? String(activeClient.class ?? "").trim() : ""
  const activeIcon = iconFromHyprClass(activeClass)

  return (
    <div className="flex max-h-[min(520px,calc(100vh-48px))] flex-col gap-3 overflow-y-auto px-3 pb-3 pt-3 text-text">
      <h2 className="text-sm font-semibold text-subtext1">Windows</h2>

      {activeClient ? (
        <div className="rounded-xl border border-teal/35 bg-surface0/90 p-3">
          <p className="mb-2 text-[10px] font-medium uppercase tracking-wide text-teal">Focused</p>
          <div className="flex gap-3">
            <span className="icon shrink-0 text-3xl text-teal">{activeIcon}</span>
            <div className="min-w-0 flex-1">
              <p className="truncate text-[13px] font-semibold leading-snug text-text">{activeTitle || activeClass || "Window"}</p>
              {activeClass ? (
                <p className="mt-0.5 truncate text-[11px] text-subtext0">{activeClass}</p>
              ) : null}
            </div>
          </div>
        </div>
      ) : (
        <p className="text-[11px] text-subtext0">No active window in IPC snapshot.</p>
      )}

      <p className="text-[11px] text-subtext0">
        {clients.length} window{clients.length === 1 ? "" : "s"} — tap a row to focus
      </p>

      <div className="flex flex-col gap-3">
        {grouped.map((g) => (
          <div key={g.wsId}>
            <p className="mb-1.5 px-0.5 text-[10px] font-semibold uppercase tracking-wide text-subtext0">
              Workspace {g.label}
            </p>
            <ul className="flex flex-col gap-1">
              {g.items.map((c) => {
                const addr = String(c.address ?? "")
                const title = String(c.title ?? "").trim() || "(no title)"
                const cls = String(c.class ?? "").trim()
                const isActive = addr === activeAddr
                const ic = iconFromHyprClass(cls)

                return (
                  <li key={addr}>
                    <button
                      type="button"
                      className={cn(
                        "flex w-full items-center gap-2 rounded-xl px-2 py-2 text-left transition-colors",
                        isActive ? "bg-teal/20 ring-1 ring-teal/40" : "bg-surface0/90 hover:bg-surface1"
                      )}
                      onClick={() => focusAddr(addr)}
                    >
                      <span className={cn("icon shrink-0 text-[22px]", isActive ? "text-teal" : "text-subtext1")}>{ic}</span>
                      <div className="min-w-0 flex-1">
                        <p className={cn("truncate text-[12px] font-medium leading-tight", isActive ? "text-teal" : "text-subtext1")}>
                          {title}
                        </p>
                        {cls ? <p className="truncate text-[10px] text-subtext0">{cls}</p> : null}
                      </div>
                      <div className="flex shrink-0 gap-1">
                        {c.floating ? (
                          <span className="rounded px-1 text-[9px] text-subtext0" title="Floating">
                            float
                          </span>
                        ) : null}
                        <span className="icon text-[18px] text-subtext0">north_east</span>
                      </div>
                    </button>
                  </li>
                )
              })}
            </ul>
          </div>
        ))}
      </div>
    </div>
  )
}
