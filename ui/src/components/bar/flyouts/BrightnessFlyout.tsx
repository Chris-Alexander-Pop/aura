import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useMemo, useState } from "react"
import api from "@/lib/api"
import { FlyoutEmpty, FlyoutLoading } from "@/components/bar/flyouts/FlyoutStates"
import {
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

      {monitorNames.length > 1 ? (
        <div>
          <FlyoutSectionLabel>Display</FlyoutSectionLabel>
          <div className="mt-0.5 flex max-h-28 flex-col gap-0.5 overflow-y-auto">
            {monitorNames.map((name) => (
              <FlyoutRadioRow
                key={name}
                label={monitorLabel(name)}
                checked={monitor === name}
                disabled={setMut.isPending}
                onSelect={() => setMonitor(name)}
              />
            ))}
          </div>
        </div>
      ) : null}

      <FlyoutSlider
        label={`Brightness (${pct}%)`}
        value={pct}
        min={5}
        max={100}
        disabled={setMut.isPending}
        accent="amber"
        onChange={(v) => setMut.mutate({ mon: monitor, percent: v / 100 })}
      />
    </FlyoutShell>
  )
}
