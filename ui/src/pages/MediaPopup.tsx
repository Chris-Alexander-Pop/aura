import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useCallback, useEffect, useMemo, useRef, useState, type PointerEvent } from "react"
import api from "@/lib/api"
import {
  createBrightnessSession,
  registerBrightnessSession,
  type BrightnessSetResult,
} from "@/lib/brightness-session"
import { brightnessIcon, volumeIcon } from "@/lib/hw-control-icons"
import { postPanelHover, scheduleLeaveAfterDrag } from "@/lib/panel-hover"
import { cn } from "@/lib/utils"
import { connectWs, useWsStore } from "@/lib/ws"

const THUMB = 14

function clampPct(n: number) {
  return Math.min(100, Math.max(0, Math.round(n)))
}

function clampBrightnessPct(n: number) {
  return Math.min(100, Math.max(1, Math.round(n)))
}

function VerticalSlider({
  value,
  accent,
  label,
  onChange,
  onCommit,
}: {
  value: number
  accent: "teal" | "yellow"
  label: string
  onChange: (pct: number) => void
  onCommit?: (pct: number) => void
}) {
  const trackRef = useRef<HTMLDivElement>(null)
  const dragging = useRef(false)
  const [active, setActive] = useState(false)

  const pctFromClientY = useCallback(
    (clientY: number) => {
      const track = trackRef.current
      if (!track) return value
      const rect = track.getBoundingClientRect()
      if (rect.height <= 0) return value
      const ratio = 1 - (clientY - rect.top) / rect.height
      return clampPct(ratio * 100)
    },
    [value],
  )

  const onPointerDown = (e: PointerEvent<HTMLDivElement>) => {
    dragging.current = true
    setActive(true)
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
    setActive(false)
    onCommit?.(pctFromClientY(e.clientY))
    try {
      e.currentTarget.releasePointerCapture(e.pointerId)
    } catch {
      /* ignore */
    }
  }

  const fillPct = clampPct(value)
  const fillColor = accent === "yellow" ? "bg-yellow" : "bg-teal"
  const glow =
    accent === "yellow"
      ? "shadow-[0_0_12px_rgb(var(--c-yellow)/0.4)]"
      : "shadow-[0_0_12px_rgb(var(--c-teal)/0.4)]"

  return (
    <div
      ref={trackRef}
      role="slider"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={fillPct}
      className="relative h-full w-3 min-h-0 shrink-0 touch-none cursor-pointer select-none rounded-full bg-base/70 ring-1 ring-inset ring-surface1/50"
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
    >
      <div
        className={cn("absolute inset-x-0 bottom-0 rounded-full", fillColor, active ? "opacity-100" : "opacity-90")}
        style={{ height: `${fillPct}%` }}
      />
      <div
        className={cn(
          "absolute left-1/2 -translate-x-1/2 rounded-full bg-text ring-2 ring-mantle",
          glow,
          active && "scale-110",
        )}
        style={{
          width: THUMB,
          height: THUMB,
          bottom: `calc(${fillPct}% - ${THUMB / 2}px)`,
        }}
      />
    </div>
  )
}

function SliderColumn({
  icon,
  label,
  value,
  accent,
  onChange,
  onCommit,
}: {
  icon: string
  label: string
  value: number
  accent: "teal" | "yellow"
  onChange: (pct: number) => void
  onCommit?: (pct: number) => void
}) {
  return (
    <div className="flex h-full min-h-0 w-8 flex-col items-center gap-2.5">
      <div className="flex min-h-0 w-full flex-1 items-center justify-center">
        <VerticalSlider
          value={value}
          accent={accent}
          label={label}
          onChange={onChange}
          onCommit={onCommit}
        />
      </div>
      <span className="icon shrink-0 text-[20px] leading-none text-subtext0" aria-hidden>
        {icon}
      </span>
    </div>
  )
}

export default function MediaPopup() {
  const qc = useQueryClient()
  const [volumePct, setVolumePct] = useState(50)
  const [brightnessPct, setBrightnessPct] = useState(50)
  const draggingVol = useRef(false)
  const draggingBright = useRef(false)

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
    if (!defaultSink || draggingVol.current) return
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
    if (draggingBright.current) return
    if (typeof brightness?.brightness === "number") {
      setBrightnessPct(clampBrightnessPct(brightness.brightness * 100))
    }
  }, [brightness?.brightness])

  const { mutateAsync: setBrightnessAsync } = useMutation({
    mutationFn: (percent: number) =>
      api.setBrightness("active", percent / 100) as Promise<BrightnessSetResult>,
  })

  const sessionRef = useRef<ReturnType<typeof createBrightnessSession> | null>(null)

  useEffect(() => {
    const session = createBrightnessSession(
      (frac) => setBrightnessAsync(frac * 100),
      (data) => {
        if (draggingBright.current) return
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
    draggingVol.current = true
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

  const commitVolume = (pct: number) => {
    applyVolume(pct)
    draggingVol.current = false
  }

  const onBrightnessChange = (pct: number) => {
    draggingBright.current = true
    const next = clampBrightnessPct(pct)
    setBrightnessPct(next)
    sessionRef.current?.setTarget(next / 100)
  }

  const onBrightnessCommit = (pct: number) => {
    const next = clampBrightnessPct(pct)
    setBrightnessPct(next)
    sessionRef.current?.setTarget(next / 100, { flush: true })
    draggingBright.current = false
  }

  return (
    <div
      className="flex h-full w-full items-stretch justify-center"
      onMouseEnter={() => postPanelHover("mediaPopupHover", true)}
      onMouseLeave={(e) =>
        scheduleLeaveAfterDrag(e, () => postPanelHover("mediaPopupHover", false))
      }
    >
      <div className="flex h-full w-full items-stretch justify-center gap-2.5 rounded-l-2xl border border-r-0 border-surface0/50 bg-mantle/95 px-3 py-3 text-text shadow-2xl backdrop-blur-xl">
        <SliderColumn
          icon={volumeIcon(volumePct, sinkMuted)}
          label={sinkMuted ? "Volume (muted)" : "Volume"}
          value={volumePct}
          accent="teal"
          onChange={applyVolume}
          onCommit={commitVolume}
        />
        <SliderColumn
          icon={brightnessIcon(brightnessPct)}
          label="Brightness"
          value={brightnessPct}
          accent="yellow"
          onChange={onBrightnessChange}
          onCommit={onBrightnessCommit}
        />
      </div>
    </div>
  )
}
