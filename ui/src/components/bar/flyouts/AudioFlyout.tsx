import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useMemo } from "react"
import api from "@/lib/api"
import { connectWs, useWsStore } from "@/lib/ws"
import { cn } from "@/lib/utils"
import { FlyoutEmpty, FlyoutLoading } from "@/components/bar/flyouts/FlyoutStates"

export default function AudioFlyout() {
  const qc = useQueryClient()

  useEffect(() => {
    connectWs()
    const off = useWsStore.getState().on("Audio.StateChanged", () => {
      void qc.invalidateQueries({ queryKey: ["audio-devices"] })
    })
    return off
  }, [qc])

  const { data, isPending, isError } = useQuery({
    queryKey: ["audio-devices"],
    queryFn: api.getAudioDevices,
    refetchInterval: 4000,
  })

  const defaultSink = useMemo(() => data?.sinks.find((s) => s.is_default) ?? data?.sinks[0], [data])
  const defaultSource = useMemo(
    () => data?.sources.find((s) => s.is_default) ?? data?.sources[0],
    [data]
  )

  const sinkVolMut = useMutation({
    mutationFn: ({ device_id, volume }: { device_id: number; volume: number }) =>
      api.setSinkVolume(device_id, volume),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const sinkMuteMut = useMutation({
    mutationFn: ({ device_id, muted }: { device_id: number; muted: boolean }) =>
      api.setSinkMute(device_id, muted),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const sourceVolMut = useMutation({
    mutationFn: ({ device_id, volume }: { device_id: number; volume: number }) =>
      api.setSourceVolume(device_id, volume),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const sourceMuteMut = useMutation({
    mutationFn: ({ device_id, muted }: { device_id: number; muted: boolean }) =>
      api.setSourceMute(device_id, muted),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const setDefaultMut = useMutation({
    mutationFn: ({ device_id, type }: { device_id: number; type: "output" | "input" }) =>
      api.setDefaultAudioDevice(device_id, type),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const busy =
    sinkVolMut.isPending ||
    sinkMuteMut.isPending ||
    sourceVolMut.isPending ||
    sourceMuteMut.isPending ||
    setDefaultMut.isPending

  if (isPending) {
    return (
      <div className="flex flex-col gap-3 px-3 pb-3 pt-3 text-text">
        <h2 className="pr-2 text-sm font-semibold leading-tight text-subtext1">Audio</h2>
        <FlyoutLoading label="Reading devices…" />
      </div>
    )
  }

  if (isError || !defaultSink) {
    return (
      <div className="flex flex-col gap-3 px-3 pb-3 pt-3 text-text">
        <h2 className="pr-2 text-sm font-semibold leading-tight text-subtext1">Audio</h2>
        <FlyoutEmpty icon="volume_off" title="No audio output" detail="Check PipeWire / WirePlumber." />
      </div>
    )
  }

  const sinkPct = Math.round(Math.min(1, Math.max(0, defaultSink.volume)) * 100)
  const sinkMuted = !!defaultSink.muted

  return (
    <div className="flex flex-col gap-3 px-3 pb-3 pt-3 text-text">
      <h2 className="pr-2 text-sm font-semibold leading-tight text-subtext1">Audio</h2>

      <div className="space-y-2">
        <div className="flex items-center justify-between gap-2">
          <p className="min-w-0 truncate text-xs font-medium text-text">{defaultSink.name}</p>
          <button
            type="button"
            disabled={busy}
            title={sinkMuted ? "Unmute" : "Mute"}
            className={cn(
              "shrink-0 rounded-lg p-1 transition-colors",
              sinkMuted ? "text-red" : "text-subtext1 hover:text-text"
            )}
            onClick={() => sinkMuteMut.mutate({ device_id: defaultSink.id, muted: !sinkMuted })}
          >
            <span className="icon text-lg">{sinkMuted ? "volume_off" : "volume_up"}</span>
          </button>
        </div>
        <input
          type="range"
          min={0}
          max={100}
          step={1}
          value={sinkPct}
          disabled={busy}
          aria-label="Output volume"
          onChange={(e) =>
            sinkVolMut.mutate({
              device_id: defaultSink.id,
              volume: Number(e.target.value) / 100,
            })
          }
          className="h-2 w-full cursor-pointer appearance-none rounded-full border border-surface1/30 bg-base/90 accent-teal"
        />
        <p className="text-[11px] tabular-nums text-subtext0">{sinkMuted ? "Muted" : `${sinkPct}%`}</p>
      </div>

      {(data?.sinks.length ?? 0) > 1 ? (
        <div className="space-y-1">
          <p className="text-[10px] uppercase tracking-wide text-subtext1">Output</p>
          <div className="flex max-h-24 flex-col gap-0.5 overflow-y-auto">
            {data!.sinks.map((s) => (
              <button
                key={s.id}
                type="button"
                disabled={busy || s.is_default}
                className={cn(
                  "rounded-lg px-2 py-1.5 text-left text-[11px] transition-colors",
                  s.is_default ? "bg-mauve/20 text-text" : "text-subtext1 hover:bg-surface0/80"
                )}
                onClick={() => setDefaultMut.mutate({ device_id: s.id, type: "output" })}
              >
                {s.name}
              </button>
            ))}
          </div>
        </div>
      ) : null}

      {defaultSource ? (
        <div className="space-y-2 border-t border-surface0/50 pt-2">
          <div className="flex items-center justify-between gap-2">
            <p className="min-w-0 truncate text-xs font-medium text-text">{defaultSource.name}</p>
            <button
              type="button"
              disabled={busy}
              title={defaultSource.muted ? "Unmute mic" : "Mute mic"}
              className={cn(
                "shrink-0 rounded-lg p-1 transition-colors",
                defaultSource.muted ? "text-red" : "text-subtext1 hover:text-text"
              )}
              onClick={() =>
                sourceMuteMut.mutate({ device_id: defaultSource.id, muted: !defaultSource.muted })
              }
            >
              <span className="icon text-lg">{defaultSource.muted ? "mic_off" : "mic"}</span>
            </button>
          </div>
          <input
            type="range"
            min={0}
            max={100}
            step={1}
            value={Math.round(Math.min(1, Math.max(0, defaultSource.volume)) * 100)}
            disabled={busy}
            aria-label="Microphone volume"
            onChange={(e) =>
              sourceVolMut.mutate({
                device_id: defaultSource.id,
                volume: Number(e.target.value) / 100,
              })
            }
            className="h-2 w-full cursor-pointer appearance-none rounded-full border border-surface1/30 bg-base/90 accent-sapphire"
          />
          <p className="text-[10px] uppercase tracking-wide text-subtext1">Input</p>
        </div>
      ) : null}

      {(data?.sources.length ?? 0) > 1 ? (
        <div className="space-y-1">
          <p className="text-[10px] uppercase tracking-wide text-subtext1">Input device</p>
          <div className="flex max-h-24 flex-col gap-0.5 overflow-y-auto">
            {data!.sources.map((s) => (
              <button
                key={s.id}
                type="button"
                disabled={busy || s.is_default}
                className={cn(
                  "rounded-lg px-2 py-1.5 text-left text-[11px] transition-colors",
                  s.is_default ? "bg-sapphire/20 text-text" : "text-subtext1 hover:bg-surface0/80"
                )}
                onClick={() => setDefaultMut.mutate({ device_id: s.id, type: "input" })}
              >
                {s.name}
              </button>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  )
}
