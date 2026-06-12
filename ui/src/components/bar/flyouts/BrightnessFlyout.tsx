import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useMemo, useState } from "react"
import api from "@/lib/api"
import { FlyoutEmpty, FlyoutLoading } from "@/components/bar/flyouts/FlyoutStates"

export default function BrightnessFlyout() {
  const qc = useQueryClient()
  const [monitor, setMonitor] = useState<string>("active")

  const { data: monitorList, isPending: listPending } = useQuery({
    queryKey: ["brightness-monitors"],
    queryFn: api.listBrightnessMonitors,
    refetchInterval: 60_000,
  })

  const monitorNames = useMemo(() => {
    const fromSidecar = (monitorList?.monitors ?? [])
      .map((m) => m.monitor)
      .filter((n): n is string => typeof n === "string" && n.length > 0)
    const unique = [...new Set(fromSidecar)]
    if (unique.length === 0) return ["active"]
    if (unique.length === 1) return unique
    return ["active", ...unique.filter((n) => n !== "active")]
  }, [monitorList])

  useEffect(() => {
    if (monitor !== "active" && !monitorNames.includes(monitor)) {
      setMonitor(monitorNames[0] ?? "active")
    }
  }, [monitor, monitorNames])

  const { data: level, isPending, isError } = useQuery({
    queryKey: ["brightness", monitor],
    queryFn: () => api.getBrightness(monitor),
    refetchInterval: 8000,
    enabled: !listPending,
  })

  const setMut = useMutation({
    mutationFn: ({ mon, percent }: { mon: string; percent: number }) =>
      api.setBrightness(mon, percent),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["brightness", monitor] })
      void qc.invalidateQueries({ queryKey: ["brightness", "active"] })
      void qc.invalidateQueries({ queryKey: ["brightness-monitors"] })
    },
  })

  const pct = Math.round((level?.brightness ?? 0.5) * 100)

  if ((isPending || listPending) && level == null) {
    return (
      <div className="flex flex-col gap-3 px-3 pb-3 pt-3 text-text">
        <h2 className="pr-2 text-sm font-semibold leading-tight text-subtext1">Brightness</h2>
        <FlyoutLoading label="Reading backlight…" />
      </div>
    )
  }

  if (isError) {
    return (
      <div className="flex flex-col gap-3 px-3 pb-3 pt-3 text-text">
        <h2 className="pr-2 text-sm font-semibold leading-tight text-subtext1">Brightness</h2>
        <FlyoutEmpty
          icon="brightness_6"
          title="Not supported"
          detail="This display may not expose DDC/CI or backlight controls."
        />
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-3 px-3 pb-3 pt-3 text-text">
      <h2 className="pr-2 text-sm font-semibold leading-tight text-subtext1">Brightness</h2>

      {monitorNames.length > 1 ? (
        <select
          className="input text-xs py-1.5"
          value={monitor}
          aria-label="Display"
          onChange={(e) => setMonitor(e.target.value)}
        >
          {monitorNames.map((name) => (
            <option key={name} value={name}>
              {name === "active" ? "Active display" : name}
            </option>
          ))}
        </select>
      ) : null}

      <div className="space-y-2">
        <input
          type="range"
          min={5}
          max={100}
          step={1}
          value={pct}
          disabled={setMut.isPending}
          aria-label="Brightness"
          aria-valuenow={pct}
          aria-valuemin={5}
          aria-valuemax={100}
          onChange={(e) =>
            setMut.mutate({ mon: monitor, percent: Number(e.target.value) / 100 })
          }
          className="h-2 w-full cursor-pointer appearance-none rounded-full border border-surface1/30 bg-base/90 accent-amber"
        />
        <p className="text-[11px] tabular-nums text-subtext0">{pct}%</p>
      </div>
    </div>
  )
}
