import { useEffect, useMemo } from "react"
import { useQuery, useQueryClient } from "@tanstack/react-query"
import api from "@/lib/api"
import { parseHyprActiveWindow, parseHyprClients, type HyprClient } from "@/lib/api-types"
import { FlyoutEmpty, FlyoutLoading } from "@/components/bar/flyouts/FlyoutStates"
import { connectWs, useWsStore } from "@/lib/ws"
import { cn } from "@/lib/utils"
import { iconFromHyprClass } from "@/components/bar/hyprWindowIcon"

const HYPRLAND_REFETCH_MS = 60_000

function wsKey(c: HyprClient): number {
  const id = c.workspace?.id
  return typeof id === "number" && Number.isFinite(id) ? id : -1
}

export default function WindowsFlyout() {
  const qc = useQueryClient()

  useEffect(() => {
    connectWs()
    const off = useWsStore.getState().on("Hyprland.StateChanged", () => {
      void qc.invalidateQueries({ queryKey: ["clients"] })
      void qc.invalidateQueries({ queryKey: ["hypr-active"] })
    })
    return off
  }, [qc])

  const {
    data: rawClients,
    isPending: clientsPending,
    isError: clientsError,
  } = useQuery({
    queryKey: ["clients"],
    queryFn: api.hyprlandGetClients,
    refetchInterval: HYPRLAND_REFETCH_MS,
  })
  const { data: rawActive, isPending: activePending } = useQuery({
    queryKey: ["hypr-active"],
    queryFn: api.hyprlandGetActiveWindow,
    refetchInterval: HYPRLAND_REFETCH_MS,
  })

  const activeAddr = parseHyprActiveWindow(rawActive)?.address ?? ""
  const clients = useMemo(() => parseHyprClients(rawClients), [rawClients])

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
        (a.title ?? "").localeCompare(b.title ?? "", undefined, { sensitivity: "base" })
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

  const headerLoading = clientsPending && rawClients === undefined

  return (
    <div className="flex max-h-[min(520px,calc(100vh-48px))] flex-col gap-3 overflow-y-auto px-3 pb-3 pt-3 text-text">
      <h2 className="pr-2 text-sm font-semibold leading-tight text-subtext1">Windows</h2>

      {headerLoading ? (
        <FlyoutLoading label="Loading window list…" />
      ) : clientsError ? (
        <FlyoutEmpty
          icon="error_outline"
          title="Could not read windows"
          detail="Check that Hyprland IPC is reachable from the sidecar."
        />
      ) : activePending && rawActive === undefined ? (
        <FlyoutLoading label="Resolving focused window…" />
      ) : activeClient ? (
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
        <FlyoutEmpty icon="crop_square" title="No focused window" detail="Focus a client or tap a row below to restore context." />
      )}

      {!headerLoading && !clientsError ? (
        <>
          <p className="px-0.5 text-[11px] leading-snug text-subtext0">
            {clients.length} window{clients.length === 1 ? "" : "s"} — tap a row to focus
          </p>

          {clients.length === 0 ? (
            <FlyoutEmpty
              icon="window"
              title="No open windows"
              detail="Launch an app — clients show here grouped by workspace."
            />
          ) : (
            <div className="flex flex-col gap-3">
              {grouped.map((g) => (
                <div key={g.wsId}>
                  <p className="mb-1.5 px-0.5 text-[10px] font-semibold uppercase tracking-wide text-subtext0">
                    Workspace {g.label}
                  </p>
                  <ul className="flex flex-col gap-1">
                    {g.items.map((c) => {
                      const addr = c.address
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
                              isActive ? "bg-teal/20 ring-1 ring-teal/40" : "bg-surface0/90 hover:bg-surface1",
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
          )}
        </>
      ) : null}
    </div>
  )
}
