import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useCallback, useEffect, useMemo, useRef, useState, type PointerEvent } from "react"
import api from "@/lib/api"
import {
  createBrightnessSession,
  registerBrightnessSession,
  type BrightnessSetResult,
} from "@/lib/brightness-session"
import { brightnessIcon, volumeIcon } from "@/lib/hw-control-icons"
import { postPanelHover } from "@/lib/panel-hover"
import { cn } from "@/lib/utils"
import { connectWs, useWsStore } from "@/lib/ws"

const SLIDER_H = 132

function clampPct(n: number) {
  return Math.min(100, Math.max(0, Math.round(n)))
}

function clampBrightnessPct(n: number) {
  return Math.min(100, Math.max(1, Math.round(n)))
}

function VerticalSlider({
  value,
  accent,
  onChange,
  onCommit,
}: {
  value: number
  accent: "lavender" | "peach"
  onChange: (pct: number) => void
  onCommit?: (pct: number) => void
}) {
  const trackRef = useRef<HTMLDivElement>(null)
  const dragging = useRef(false)

  const pctFromClientY = useCallback((clientY: number) => {
    const track = trackRef.current
    if (!track) return value
    const rect = track.getBoundingClientRect()
    if (rect.height <= 0) return value
    const ratio = 1 - (clientY - rect.top) / rect.height
    return clampPct(ratio * 100)
  }, [value])

  const onPointerDown = (e: PointerEvent<HTMLDivElement>) => {
    dragging.current = true
    e.currentTarget.setPointerCapture(e.pointerId)
    const next = pctFromClientY(e.clientY)
    onChange(next)
    onCommit?.(next)
  }

  const onPointerMove = (e: PointerEvent<HTMLDivElement>) => {
    if (!dragging.current) return
    onChange(pctFromClientY(e.clientY))
  }

  const onPointerUp = (e: PointerEvent<HTMLDivElement>) => {
    if (!dragging.current) return
    dragging.current = false
    onCommit?.(pctFromClientY(e.clientY))
    try {
      e.currentTarget.releasePointerCapture(e.pointerId)
    } catch {
      /* ignore */
    }
  }

  const fillPct = clampPct(value)
  const fillColor = accent === "lavender" ? "bg-lavender/90" : "bg-peach/90"
  const thumbColor = accent === "lavender" ? "bg-lavender" : "bg-peach"

  return (
    <div
      ref={trackRef}
      role="slider"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={fillPct}
      className="relative w-2.5 shrink-0 touch-none cursor-pointer select-none rounded-full bg-surface0/45"
      style={{ height: SLIDER_H }}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
    >
      <div
        className={cn("absolute inset-x-0 bottom-0 rounded-full transition-[height] duration-75", fillColor)}
        style={{ height: `${fillPct}%` }}
      />
      <div
        className={cn(
          "absolute left-1/2 h-2 w-2 -translate-x-1/2 rounded-full shadow-sm ring-2 ring-crust/70",
          thumbColor,
        )}
        style={{ bottom: `calc(${fillPct}% - 4px)` }}
      />
    </div>
  )
}

function SliderColumn({
  icon,
  value,
  accent,
  onChange,
  onCommit,
}: {
  icon: string
  value: number
  accent: "lavender" | "peach"
  onChange: (pct: number) => void
  onCommit?: (pct: number) => void
}) {
  return (
    <div className="flex flex-col items-center gap-2">
      <VerticalSlider value={value} accent={accent} onChange={onChange} onCommit={onCommit} />
      <span className="icon text-[18px] leading-none text-subtext0">{icon}</span>
    </div>
  )
}

export default function MediaPopup() {
  const qc = useQueryClient()
  const [volumePct, setVolumePct] = useState(50)
  const [brightnessPct, setBrightnessPct] = useState(50)

  useEffect(() => {
    connectWs()
    const off = useWsStore.getState().on("Audio.StateChanged", () => {
      void qc.invalidateQueries({ queryKey: ["audio-devices"] })
    })
    return off
  }, [qc])

  const { data: audio } = useQuery({
    queryKey: ["audio-devices"],
    queryFn: api.getAudioDevices,
    refetchInterval: 4000,
  })

  const defaultSink = useMemo(
    () => audio?.sinks.find((s) => s.is_default) ?? audio?.sinks[0],
    [audio],
  )

  const sinkMuted = !!defaultSink?.muted

  useEffect(() => {
    if (!defaultSink) return
    setVolumePct(clampPct(defaultSink.volume * 100))
  }, [defaultSink])

  const sinkVolMut = useMutation({
    mutationFn: ({ device_id, volume }: { device_id: number; volume: number }) =>
      api.setSinkVolume(device_id, volume),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const unmuteMut = useMutation({
    mutationFn: (device_id: number) => api.setSinkMute(device_id, false),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const { data: brightness } = useQuery({
    queryKey: ["brightness", "active"],
    queryFn: () => api.getBrightness("active"),
    staleTime: 30_000,
  })

  useEffect(() => {
    if (typeof brightness?.brightness === "number") {
      setBrightnessPct(clampBrightnessPct(brightness.brightness * 100))
    }
  }, [brightness?.brightness])

  const { mutateAsync: setBrightnessAsync } = useMutation({
    mutationFn: (percent: number) => api.setBrightness("active", percent / 100) as Promise<BrightnessSetResult>,
  })

  const sessionRef = useRef<ReturnType<typeof createBrightnessSession> | null>(null)

  useEffect(() => {
    const session = createBrightnessSession(
      (frac) => setBrightnessAsync(frac * 100),
      (data) => {
        if (typeof data.brightness === "number") {
          setBrightnessPct(clampPct(data.brightness * 100))
        }
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

  const applyVolume = (pct: number) => {
    setVolumePct(pct)
    if (!defaultSink) return
    if (sinkMuted && pct > 0) {
      void unmuteMut.mutateAsync(defaultSink.id).then(() => {
        sinkVolMut.mutate({ device_id: defaultSink.id, volume: pct / 100 })
      })
      return
    }
    sinkVolMut.mutate({ device_id: defaultSink.id, volume: pct / 100 })
  }

  const onBrightnessChange = (pct: number) => {
    const next = clampBrightnessPct(pct)
    setBrightnessPct(next)
    sessionRef.current?.setTarget(next / 100)
  }

  const onBrightnessCommit = (pct: number) => {
    const next = clampBrightnessPct(pct)
    setBrightnessPct(next)
    sessionRef.current?.setTarget(next / 100, { flush: true })
  }

  return (
    <div
      className="flex h-full w-full items-center justify-center p-1"
      onMouseEnter={() => postPanelHover("mediaPopupHover", true)}
      onMouseLeave={() => postPanelHover("mediaPopupHover", false)}
    >
      <div className="flex h-full w-full items-end justify-center gap-4 rounded-2xl border border-surface0/60 bg-mantle/95 px-3 pb-3 pt-2.5 text-text shadow-2xl backdrop-blur-xl">
        <SliderColumn
          icon={volumeIcon(volumePct, sinkMuted)}
          value={volumePct}
          accent="lavender"
          onChange={applyVolume}
          onCommit={applyVolume}
        />
        <SliderColumn
          icon={brightnessIcon(brightnessPct)}
          value={brightnessPct}
          accent="peach"
          onChange={onBrightnessChange}
          onCommit={onBrightnessCommit}
        />
      </div>
    </div>
  )
}
