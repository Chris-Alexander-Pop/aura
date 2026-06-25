import { useMemo } from "react"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { parseHyprActiveWindow, parseHyprClients, type HyprClient } from "@/lib/api-types"
import { FlyoutEmpty, FlyoutLoading } from "@/components/bar/flyouts/FlyoutStates"
import {
  FlyoutList,
  FlyoutMeta,
  FlyoutRow,
  FlyoutRowIcon,
  FlyoutRowLabel,
  FlyoutShell,
  FlyoutTitle,
} from "@/components/bar/flyouts/FlyoutPrimitives"
import { cn } from "@/lib/utils"
import { iconFromHyprClass } from "@/components/bar/hyprWindowIcon"
import { hyprlandQueryDefaults, useHyprlandSync } from "@/lib/useHyprlandSync"

function wsKey(c: HyprClient): number {
  const id = c.workspace?.id
  return typeof id === "number" && Number.isFinite(id) ? id : -1
}

export default function WindowsFlyout() {
  useHyprlandSync()

  const {
    data: rawClients,
    isPending: clientsPending,
    isError: clientsError,
  } = useQuery({
    queryKey: ["clients"],
    queryFn: api.hyprlandGetClients,
    ...hyprlandQueryDefaults,
  })
  const { data: rawActive, isPending: activePending } = useQuery({
    queryKey: ["hypr-active"],
    queryFn: api.hyprlandGetActiveWindow,
    ...hyprlandQueryDefaults,
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
    <FlyoutShell className="max-h-[min(440px,calc(100vh-48px))] overflow-y-auto">
      <FlyoutTitle>Windows</FlyoutTitle>

      {headerLoading ? (
        <FlyoutLoading label="Loading window list…" />
      ) : clientsError ? (
        <FlyoutEmpty
          icon="error_outline"
          title="Could not read windows"
          detail="Check Hyprland IPC from the sidecar."
        />
      ) : activePending && rawActive === undefined ? (
        <FlyoutLoading label="Resolving focused window…" />
      ) : activeClient ? (
        <button
          type="button"
          className="flex min-h-8 w-full items-center gap-2 rounded-lg px-1 py-1 text-left hover:bg-surface0/60"
          onClick={() => focusAddr(activeClient.address)}
        >
          <FlyoutRowIcon icon={activeIcon} active />
          <div className="min-w-0 flex-1">
            <p className="truncate text-[11px] font-medium text-teal">{activeTitle || activeClass || "Window"}</p>
            {activeClass ? (
              <p className="truncate text-[10px] text-subtext0">{activeClass}</p>
            ) : null}
          </div>
        </button>
      ) : (
        <FlyoutEmpty icon="crop_square" title="No focused window" />
      )}

      {!headerLoading && !clientsError ? (
        <>
          <FlyoutMeta>
            {clients.length} window{clients.length === 1 ? "" : "s"} — tap to focus
          </FlyoutMeta>

          {clients.length === 0 ? (
            <FlyoutEmpty icon="window" title="No open windows" />
          ) : (
            <div className="flex flex-col gap-2">
              {grouped.map((g) => (
                <div key={g.wsId}>
                  <p className="mb-0.5 text-[9px] font-semibold uppercase tracking-wide text-subtext0">
                    WS {g.label}
                  </p>
                  <FlyoutList className="max-h-none">
                    {g.items.map((c) => {
                      const addr = c.address
                      const title = String(c.title ?? "").trim() || "(no title)"
                      const cls = String(c.class ?? "").trim()
                      const isActive = addr === activeAddr
                      const ic = iconFromHyprClass(cls)

                      return (
                        <FlyoutRow key={addr}>
                          <button
                            type="button"
                            className={cn(
                              "flex min-h-8 w-full items-center gap-1.5 rounded-lg px-1 py-0.5 text-left transition-colors",
                              isActive ? "bg-teal/15" : "hover:bg-surface0/60",
                            )}
                            onClick={() => focusAddr(addr)}
                          >
                            <FlyoutRowIcon icon={ic} active={isActive} />
                            <FlyoutRowLabel active={isActive}>{title}</FlyoutRowLabel>
                          </button>
                        </FlyoutRow>
                      )
                    })}
                  </FlyoutList>
                </div>
              ))}
            </div>
          )}
        </>
      ) : null}
    </FlyoutShell>
  )
}
