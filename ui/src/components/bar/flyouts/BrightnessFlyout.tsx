import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useCallback, useEffect, useMemo, useState } from "react"
import api from "@/lib/api"
import { FlyoutEmpty, FlyoutLoading } from "@/components/bar/flyouts/FlyoutStates"
import {
  FlyoutBanner,
  FlyoutRadioRow,
  FlyoutSectionLabel,
  FlyoutShell,
  FlyoutSlider,
  FlyoutTitle,
} from "@/components/bar/flyouts/FlyoutPrimitives"

function monitorLabel(name: string): string {
  if (name === "active") return "Active display"
  return name.replace(/_/g, " ")
}

function brightnessForMonitor(
  monitorList: { monitors?: Array<{ monitor: string; brightness: number }> } | undefined,
  name: string,
): { brightness: number } | undefined {
  const hit = monitorList?.monitors?.find((m) => m.monitor === name)
  return hit ? { brightness: hit.brightness } : undefined
}

export default function BrightnessFlyout() {
  const qc = useQueryClient()
  const [monitor, setMonitor] = useState<string>("active")
  const [setError, setSetError] = useState<string | null>(null)

  const { data: monitorList, isLoading: monitorsLoading } = useQuery({
    queryKey: ["brightness-monitors"],
    queryFn: api.listBrightnessMonitors,
    staleTime: 60_000,
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

  const { data: level, isLoading: levelLoading, isError } = useQuery({
    queryKey: ["brightness", monitor],
    queryFn: () => api.getBrightness(monitor),
    staleTime: 30_000,
    placeholderData: () =>
      brightnessForMonitor(monitorList, monitor) ??
      qc.getQueryData<{ brightness: number }>(["brightness", monitor]),
  })

  const { mutate: setBrightness } = useMutation({
    mutationFn: ({ mon, percent }: { mon: string; percent: number }) =>
      api.setBrightness(mon, percent),
    onSuccess: (data) => {
      setSetError(null)
      const row = data as { brightness?: number; monitor?: string }
      if (typeof row.brightness !== "number") return
      const key = row.monitor ?? monitor
      qc.setQueryData(["brightness", key], { brightness: row.brightness })
      if (key !== monitor) {
        qc.setQueryData(["brightness", monitor], { brightness: row.brightness })
      }
      if (monitor === "active") {
        qc.setQueryData(["brightness", "active"], { brightness: row.brightness })
      }
    },
    onError: (err) => {
      setSetError(err instanceof Error ? err.message : "Could not set brightness")
    },
  })

  const applyBrightness = useCallback(
    (v: number) => {
      const frac = v / 100
      setSetError(null)
      qc.setQueryData(["brightness", monitor], { brightness: frac })
      if (monitor === "active") {
        qc.setQueryData(["brightness", "active"], { brightness: frac })
      }
      setBrightness({ mon: monitor, percent: frac })
    },
    [monitor, qc, setBrightness],
  )

  const pct = Math.round((level?.brightness ?? 0.5) * 100)
  const bootstrapping = monitorList == null && level == null

  if (bootstrapping && (monitorsLoading || levelLoading)) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Brightness</FlyoutTitle>
        <FlyoutLoading label="Reading backlight…" />
      </FlyoutShell>
    )
  }

  if (isError) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Brightness</FlyoutTitle>
        <FlyoutEmpty
          icon="brightness_6"
          title="Not supported"
          detail="This display may not expose backlight controls."
        />
      </FlyoutShell>
    )
  }

  return (
    <FlyoutShell>
      <FlyoutTitle>Brightness</FlyoutTitle>

      {setError ? <FlyoutBanner tone="error">{setError}</FlyoutBanner> : null}

      {monitorNames.length > 1 ? (
        <div>
          <FlyoutSectionLabel>Display</FlyoutSectionLabel>
          <div className="mt-0.5 flex max-h-28 flex-col gap-0.5 overflow-y-auto">
            {monitorNames.map((name) => (
              <FlyoutRadioRow
                key={name}
                label={monitorLabel(name)}
                checked={monitor === name}
                onSelect={() => setMonitor(name)}
              />
            ))}
          </div>
        </div>
      ) : null}

      <FlyoutSlider
        label="Brightness"
        value={pct}
        min={0}
        max={100}
        live
        accent="amber"
        onChange={applyBrightness}
      />
    </FlyoutShell>
  )
}
