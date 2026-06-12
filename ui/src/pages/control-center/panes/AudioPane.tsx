import { motion } from "framer-motion"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useMemo, useState } from "react"
import api from "@/lib/api"
import { cn } from "@/lib/utils"
import { getNavItem } from "../navigation"

type AudioSink = { id: number; name: string; volume: number; muted?: boolean; is_default: boolean }
type AudioSource = AudioSink
type AudioStream = { id: number; name: string; app: string; volume: number; sink_id: number; muted?: boolean }

function DeviceMeterCard({
  kind,
  device,
  onMuteToggle,
  onVolumeCommit,
  onSetDefault,
  busy,
}: {
  kind: "output" | "input"
  device: AudioSink | AudioSource
  onMuteToggle?: (deviceId: number, muted: boolean) => void
  onVolumeCommit?: (deviceId: number, volume: number) => void
  onSetDefault?: (deviceId: number) => void
  busy?: boolean
}) {
  const pct = Math.round(Math.min(1, Math.max(0, device.volume)) * 100)
  const muted = !!device.muted

  return (
    <div
      className={cn(
        "glass-card p-4 flex flex-col gap-3 transition-colors",
        device.is_default && "border-mauve/35 shadow-[inset_0_0_0_1px_rgba(203,166,247,0.12)]"
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-center gap-2 min-w-0">
          <span className={cn("icon shrink-0", kind === "output" ? "text-teal" : "text-sapphire")}>
            {kind === "output" ? "speaker_group" : "mic"}
          </span>
          <div className="min-w-0">
            <p className="text-sm font-medium text-text truncate">{device.name}</p>
            <p className="text-[11px] text-subtext0 uppercase tracking-wide">
              {kind === "output" ? "Playback" : "Capture"}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-1.5 shrink-0">
          {onMuteToggle ? (
            <button
              type="button"
              disabled={busy}
              title={muted ? "Unmute" : "Mute"}
              className={cn(
                "flex h-8 w-8 items-center justify-center rounded-full transition-colors",
                muted ? "bg-red/20 text-red" : "bg-surface0/80 text-subtext1 hover:text-text"
              )}
              onClick={() => onMuteToggle(device.id, !muted)}
            >
              <span className="icon text-lg">{muted ? "volume_off" : "volume_up"}</span>
            </button>
          ) : null}
          {device.is_default && (
            <span className="text-[10px] font-semibold uppercase tracking-wider text-mauve bg-mauve/15 px-2 py-0.5 rounded-full">
              Default
            </span>
          )}
          {!device.is_default && onSetDefault ? (
            <button
              type="button"
              disabled={busy}
              className="text-[10px] font-semibold uppercase tracking-wider text-subtext0 hover:text-mauve px-2 py-0.5 rounded-full border border-surface1/50"
              onClick={() => onSetDefault(device.id)}
            >
              Set default
            </button>
          ) : null}
        </div>
      </div>
      <div className="space-y-1.5">
        <div className="flex justify-between text-[11px] text-subtext1">
          <span>Level</span>
          <span className="tabular-nums text-subtext0">{muted ? "Muted" : `${pct}%`}</span>
        </div>
        {onVolumeCommit ? (
          <input
            type="range"
            min={0}
            max={100}
            step={1}
            value={pct}
            disabled={busy}
            onChange={(e) => onVolumeCommit(device.id, Number(e.target.value) / 100)}
            className="w-full h-2 rounded-full appearance-none cursor-pointer accent-teal bg-base/90 border border-surface1/30"
          />
        ) : (
          <div className="h-2 rounded-full bg-base/80 overflow-hidden border border-surface1/30">
            <div
              className={cn(
                "h-full rounded-full transition-[width] duration-300 ease-out",
                kind === "output"
                  ? "bg-gradient-to-r from-teal/70 via-teal to-sky/90"
                  : "bg-gradient-to-r from-sapphire/70 via-blue to-lavender/90"
              )}
              style={{ width: `${pct}%` }}
            />
          </div>
        )}
      </div>
    </div>
  )
}

function StreamMixerCard({
  stream,
  sinkLabel,
  volumeBusy,
  muteBusy,
  onVolumeCommit,
  onMuteToggle,
  muted,
}: {
  stream: AudioStream
  sinkLabel: string
  volumeBusy: boolean
  muteBusy: boolean
  onVolumeCommit: (streamId: number, ratio: number) => void
  onMuteToggle: (streamId: number, next: boolean) => void
  muted: boolean
}) {
  const [localPct, setLocalPct] = useState(stream.volume)

  useEffect(() => {
    setLocalPct(stream.volume)
  }, [stream.id, stream.volume])

  const displayTitle = stream.name.trim() || stream.app || "Audio stream"

  return (
    <div className="glass-card p-4 rounded-2xl flex flex-col gap-3 border-surface1/35 hover:border-surface2/50 transition-colors">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <p className="text-sm font-semibold text-text truncate">{displayTitle}</p>
          <p className="text-xs text-subtext0 truncate">{stream.app || "Unknown application"}</p>
          <p className="text-[11px] text-subtext1 mt-1 truncate">
            <span className="icon text-[14px] align-middle mr-1 text-subtext0">output</span>
            {sinkLabel}
          </p>
        </div>
        <button
          type="button"
          disabled={muteBusy}
          onClick={() => onMuteToggle(stream.id, !muted)}
          className={cn(
            "icon-btn rounded-xl shrink-0 border transition-colors",
            muted
              ? "border-red/40 bg-red/10 text-red hover:bg-red/15"
              : "border-surface1/50 bg-surface0/40 text-subtext1 hover:text-text hover:bg-surface1/40"
          )}
          title={muted ? "Unmute" : "Mute"}
        >
          <span className="icon text-xl">{muted ? "volume_off" : "volume_up"}</span>
        </button>
      </div>

      <div className="space-y-2">
        <div className="flex justify-between items-baseline">
          <span className="text-[11px] uppercase tracking-wider text-subtext1">Volume</span>
          <span
            className={cn(
              "text-xs tabular-nums font-medium",
              volumeBusy ? "text-subtext0 animate-pulse" : "text-subtext1"
            )}
          >
            {localPct}%
          </span>
        </div>
        <input
          type="range"
          min={0}
          max={100}
          step={1}
          value={localPct}
          disabled={volumeBusy}
          onChange={(e) => setLocalPct(Number(e.target.value))}
          onPointerUp={() => onVolumeCommit(stream.id, localPct / 100)}
          onKeyUp={(e) => {
            if (e.key === "Enter") onVolumeCommit(stream.id, localPct / 100)
          }}
          className={cn(
            "w-full h-2 rounded-full appearance-none cursor-pointer accent-mauve",
            "bg-base/90 border border-surface1/40",
            "[&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:w-4",
            "[&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-mauve [&::-webkit-slider-thumb]:shadow-lg [&::-webkit-slider-thumb]:shadow-mauve/25",
            "[&::-webkit-slider-thumb]:border [&::-webkit-slider-thumb]:border-mantle",
            "[&::-moz-range-thumb]:h-4 [&::-moz-range-thumb]:w-4 [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:bg-mauve [&::-moz-range-thumb]:border-0"
          )}
        />
      </div>
    </div>
  )
}

export function AudioPane() {
  const queryClient = useQueryClient()
  const { icon, label } = getNavItem("audio")

  const devicesQuery = useQuery({
    queryKey: ["audio-devices"],
    queryFn: api.getAudioDevices,
    refetchInterval: 2500,
  })

  const streamsQuery = useQuery({
    queryKey: ["audio-streams"],
    queryFn: api.getAudioStreams,
    refetchInterval: 1500,
  })

  const sinkNames = useMemo(() => {
    const map = new Map<number, string>()
    for (const s of devicesQuery.data?.sinks ?? []) {
      map.set(s.id, s.name)
    }
    return map
  }, [devicesQuery.data?.sinks])

  const volumeMutation = useMutation({
    mutationFn: ({ stream_id, volume }: { stream_id: number; volume: number }) =>
      api.setStreamVolume(stream_id, volume),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["audio-streams"] })
    },
  })

  const muteMutation = useMutation({
    mutationFn: ({ stream_id, muted }: { stream_id: number; muted: boolean }) =>
      api.setStreamMute(stream_id, muted),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["audio-streams"] })
    },
  })

  const sinkMuteMutation = useMutation({
    mutationFn: ({ device_id, muted }: { device_id: number; muted: boolean }) =>
      api.setSinkMute(device_id, muted),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const sourceMuteMutation = useMutation({
    mutationFn: ({ device_id, muted }: { device_id: number; muted: boolean }) =>
      api.setSourceMute(device_id, muted),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const sinkVolumeMutation = useMutation({
    mutationFn: ({ device_id, volume }: { device_id: number; volume: number }) =>
      api.setSinkVolume(device_id, volume),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const sourceVolumeMutation = useMutation({
    mutationFn: ({ device_id, volume }: { device_id: number; volume: number }) =>
      api.setSourceVolume(device_id, volume),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const setDefaultMutation = useMutation({
    mutationFn: ({ device_id, type }: { device_id: number; type: "output" | "input" }) =>
      api.setDefaultAudioDevice(device_id, type),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const refreshMutation = useMutation({
    mutationFn: api.refreshAudio,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["audio-devices"] })
      await queryClient.invalidateQueries({ queryKey: ["audio-streams"] })
    },
  })

  const deviceBusy =
    sinkMuteMutation.isPending ||
    sourceMuteMutation.isPending ||
    sinkVolumeMutation.isPending ||
    sourceVolumeMutation.isPending ||
    setDefaultMutation.isPending

  const sinks = devicesQuery.data?.sinks ?? []
  const sources = devicesQuery.data?.sources ?? []
  const streams = streamsQuery.data ?? []

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-6 p-6 h-full overflow-y-auto"
    >
      <header className="flex items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-3 mb-1">
            <span className="icon text-mauve text-2xl">{icon}</span>
            <h2 className="text-xl font-semibold text-text">{label}</h2>
          </div>
          <p className="text-xs text-subtext1 max-w-prose leading-relaxed">
            Outputs, inputs, and per-application streams. Default device mute and volume use PipeWire via the sidecar.
          </p>
        </div>
        <button
          type="button"
          disabled={refreshMutation.isPending}
          className="btn-surface text-xs shrink-0 flex items-center gap-1.5"
          onClick={() => refreshMutation.mutate()}
        >
          <span className={cn("icon text-base", refreshMutation.isPending && "animate-spin")}>refresh</span>
          {refreshMutation.isPending ? "Refreshing…" : "Refresh"}
        </button>
      </header>

      <section className="space-y-3">
        <h3 className="text-[11px] font-semibold uppercase tracking-[0.14em] text-subtext0">Output devices</h3>
        {devicesQuery.isLoading ? (
          <div className="grid gap-3 sm:grid-cols-2">
            <div className="skeleton h-28 rounded-2xl" />
          </div>
        ) : sinks.length === 0 ? (
          <p className="text-sm text-subtext0">No playback devices reported.</p>
        ) : (
          <div className="grid gap-3 sm:grid-cols-2">
            {sinks.map((d) => (
              <DeviceMeterCard
                key={`sink-${d.id}`}
                kind="output"
                device={d}
                busy={deviceBusy}
                onMuteToggle={(id, muted) => sinkMuteMutation.mutate({ device_id: id, muted })}
                onVolumeCommit={(id, vol) => sinkVolumeMutation.mutate({ device_id: id, volume: vol })}
                onSetDefault={
                  d.is_default
                    ? undefined
                    : (id) => setDefaultMutation.mutate({ device_id: id, type: "output" })
                }
              />
            ))}
          </div>
        )}
      </section>

      <section className="space-y-3">
        <h3 className="text-[11px] font-semibold uppercase tracking-[0.14em] text-subtext0">Input devices</h3>
        {devicesQuery.isLoading ? (
          <div className="grid gap-3 sm:grid-cols-2">
            <div className="skeleton h-28 rounded-2xl" />
          </div>
        ) : sources.length === 0 ? (
          <p className="text-sm text-subtext0">No capture devices reported.</p>
        ) : (
          <div className="grid gap-3 sm:grid-cols-2">
            {sources.map((d) => (
              <DeviceMeterCard
                key={`src-${d.id}`}
                kind="input"
                device={d}
                busy={deviceBusy}
                onMuteToggle={(id, muted) => sourceMuteMutation.mutate({ device_id: id, muted })}
                onVolumeCommit={(id, vol) => sourceVolumeMutation.mutate({ device_id: id, volume: vol })}
                onSetDefault={
                  d.is_default
                    ? undefined
                    : (id) => setDefaultMutation.mutate({ device_id: id, type: "input" })
                }
              />
            ))}
          </div>
        )}
      </section>

      <section className="space-y-3">
        <h3 className="text-[11px] font-semibold uppercase tracking-[0.14em] text-subtext0">
          Application streams
        </h3>
        {streamsQuery.isLoading ? (
          <div className="flex flex-col gap-3">
            <div className="skeleton h-36 rounded-2xl" />
            <div className="skeleton h-36 rounded-2xl" />
          </div>
        ) : streams.length === 0 ? (
          <div className="glass-card p-8 rounded-2xl text-center border-dashed border-surface1/50">
            <span className="icon text-4xl text-subtext0 mb-2 block">graphic_eq</span>
            <p className="text-sm text-subtext1">No active playback streams</p>
            <p className="text-xs text-subtext0 mt-1">Start audio in an app to mix it here.</p>
          </div>
        ) : (
          <div className="flex flex-col gap-3">
            {streams.map((s) => (
              <StreamMixerCard
                key={s.id}
                stream={s}
                sinkLabel={sinkNames.get(s.sink_id) ?? `Sink ${s.sink_id}`}
                volumeBusy={
                  volumeMutation.isPending && volumeMutation.variables?.stream_id === s.id
                }
                muteBusy={muteMutation.isPending && muteMutation.variables?.stream_id === s.id}
                muted={!!s.muted}
                onVolumeCommit={(streamId, ratio) => volumeMutation.mutate({ stream_id: streamId, volume: ratio })}
                onMuteToggle={(streamId, next) => muteMutation.mutate({ stream_id: streamId, muted: next })}
              />
            ))}
          </div>
        )}
      </section>

      {(devicesQuery.isError || streamsQuery.isError) && (
        <p className="text-xs text-red">
          {[devicesQuery.error, streamsQuery.error]
            .filter((e): e is Error => e instanceof Error)
            .map((e) => e.message)
            .join(" · ") || "Could not load audio state."}
        </p>
      )}
    </motion.div>
  )
}
