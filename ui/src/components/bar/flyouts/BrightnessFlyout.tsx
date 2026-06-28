import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import api from "@/lib/api"
import {
  createBrightnessSession,
  registerBrightnessSession,
  type BrightnessSetResult,
} from "@/lib/brightness-session"
import { FlyoutEmpty, FlyoutLoading } from "@/components/bar/flyouts/FlyoutStates"
import {
  FlyoutBanner,
  FlyoutRadioRow,
  FlyoutSectionLabel,
  FlyoutShell,
  FlyoutSlider,
  FlyoutTitle,
} from "@/components/bar/flyouts/FlyoutPrimitives"

type BrightnessRow = { brightness: number; monitor?: string }

function monitorLabel(name: string): string {
  if (name === "active") return "Active display"
  return name.replace(/_/g, " ")
}

function brightnessForMonitor(
  monitorList: { monitors?: Array<{ monitor: string; brightness: number }> } | undefined,
  name: string,
): BrightnessRow | undefined {
  const hit = monitorList?.monitors?.find((m) => m.monitor === name)
  return hit ? { brightness: hit.brightness, monitor: hit.monitor } : undefined
}

export default function BrightnessFlyout() {
  const qc = useQueryClient()
  const [monitor, setMonitor] = useState<string>("active")
  const [setError, setSetError] = useState<string | null>(null)
  const monitorRef = useRef(monitor)
  monitorRef.current = monitor

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

  const { data: level, isLoading: levelLoading, isError } = useQuery<BrightnessRow>({
    queryKey: ["brightness", monitor],
    queryFn: () => api.getBrightness(monitor),
    staleTime: 30_000,
    placeholderData: () =>
      brightnessForMonitor(monitorList, monitor) ??
      qc.getQueryData<BrightnessRow>(["brightness", monitor]),
  })

  const writeOptimistic = useCallback(
    (frac: number, monitorName?: string) => {
      setSetError(null)
      const row: BrightnessRow = { brightness: frac, monitor: monitorName }
      const key = monitorRef.current
      qc.setQueryData(["brightness", key], row)
      if (key === "active") {
        qc.setQueryData(["brightness", "active"], row)
      }
    },
    [qc],
  )

  const applyBrightnessSuccess = useCallback(
    (data: BrightnessSetResult) => {
      setSetError(null)
      if (typeof data.brightness !== "number") return
      const key = monitorRef.current
      const resolvedMonitor = data.monitor ?? key
      const cached: BrightnessRow = { brightness: data.brightness, monitor: resolvedMonitor }
      qc.setQueryData(["brightness", resolvedMonitor], cached)
      if (resolvedMonitor !== key) {
        qc.setQueryData(["brightness", key], cached)
      }
      if (key === "active") {
        qc.setQueryData(["brightness", "active"], cached)
      }
    },
    [qc],
  )

  const { mutateAsync: setBrightnessAsync } = useMutation({
    mutationFn: ({ mon, percent }: { mon: string; percent: number }) =>
      api.setBrightness(mon, percent) as Promise<BrightnessSetResult>,
  })

  const onSuccessRef = useRef(applyBrightnessSuccess)
  onSuccessRef.current = applyBrightnessSuccess

  const sessionRef = useRef<ReturnType<typeof createBrightnessSession> | null>(null)

  useEffect(() => {
    const session = createBrightnessSession(
      (frac) => setBrightnessAsync({ mon: monitorRef.current, percent: frac }),
      (data) => onSuccessRef.current(data),
      (err) => {
        setSetError(err instanceof Error ? err.message : "Could not set brightness")
      },
    )
    sessionRef.current = session
    registerBrightnessSession(session)
    return () => {
      session.dispose()
      registerBrightnessSession(null)
      sessionRef.current = null
    }
  }, [setBrightnessAsync])

  const applyLiveBrightness = useCallback(
    (v: number) => {
      const frac = v / 100
      writeOptimistic(frac, level?.monitor)
      sessionRef.current?.setTarget(frac)
    },
    [level?.monitor, writeOptimistic],
  )

  const applyFinalBrightness = useCallback(
    (v: number) => {
      const frac = v / 100
      writeOptimistic(frac, level?.monitor)
      sessionRef.current?.setTarget(frac, { flush: true })
    },
    [level?.monitor, writeOptimistic],
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
        showThumb={false}
        onLiveChange={applyLiveBrightness}
        onChange={applyFinalBrightness}
      />
    </FlyoutShell>
  )
}
