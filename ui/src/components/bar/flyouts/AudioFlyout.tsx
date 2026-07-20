import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useMemo } from "react"
import api from "@/lib/api"
import { connectWs, useWsStore } from "@/lib/ws"
import { cn } from "@/lib/utils"
import { FlyoutEmpty, FlyoutLoading } from "@/components/bar/flyouts/FlyoutStates"
import {
  FlyoutExpandLink,
  FlyoutIconButton,
  FlyoutRadioRow,
  FlyoutSectionLabel,
  FlyoutShell,
  FlyoutSlider,
  FlyoutTitle,
} from "@/components/bar/flyouts/FlyoutPrimitives"

function volumeIcon(pct: number, muted: boolean): string {
  if (muted || pct <= 0) return "volume_off"
  if (pct <= 30) return "volume_mute"
  if (pct <= 65) return "volume_down"
  return "volume_up"
}

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

  const setDefaultMut = useMutation({
    mutationFn: ({ device_id, type }: { device_id: number; type: "output" | "input" }) =>
      api.setDefaultAudioDevice(device_id, type),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["audio-devices"] }),
  })

  const busy = sinkVolMut.isPending || setDefaultMut.isPending || sinkMuteMut.isPending

  if (isPending) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Audio</FlyoutTitle>
        <FlyoutLoading label="Reading devices…" />
      </FlyoutShell>
    )
  }

  if (isError || !defaultSink) {
    return (
      <FlyoutShell>
        <FlyoutTitle>Audio</FlyoutTitle>
        <FlyoutEmpty icon="volume_off" title="No audio output" detail="Check PipeWire / WirePlumber." />
      </FlyoutShell>
    )
  }

  const sinkPct = Math.round(Math.min(1, Math.max(0, defaultSink.volume)) * 100)
  const sinkMuted = !!defaultSink.muted

  return (
    <FlyoutShell className="gap-2">
      <FlyoutTitle>Audio</FlyoutTitle>

      <FlyoutSlider
        label={sinkMuted ? "Muted" : "Volume"}
        icon={volumeIcon(sinkPct, sinkMuted)}
        value={sinkPct}
        disabled={sinkMuted}
        live
        liveDebounceMs={0}
        trailing={
          <FlyoutIconButton
            icon={sinkMuted ? "volume_off" : "volume_up"}
            active={sinkMuted}
            disabled={busy}
            title={sinkMuted ? "Unmute" : "Mute"}
            onClick={() =>
              sinkMuteMut.mutate({ device_id: defaultSink.id, muted: !sinkMuted })
            }
          />
        }
        onChange={(v) =>
          sinkVolMut.mutate({
            device_id: defaultSink.id,
            volume: v / 100,
          })
        }
      />

      <div className="max-h-32 overflow-y-auto">
        <FlyoutSectionLabel>Output</FlyoutSectionLabel>
        <div className="mt-1 flex flex-col gap-0.5">
          {data!.sinks.map((s) => (
            <FlyoutRadioRow
              key={s.id}
              label={s.name}
              checked={s.is_default}
              disabled={busy}
              onSelect={() => setDefaultMut.mutate({ device_id: s.id, type: "output" })}
            />
          ))}
        </div>
      </div>

      {(data?.sources.length ?? 0) > 0 ? (
        <div className={cn("max-h-28 overflow-y-auto border-t border-surface0/50 pt-2")}>
          <FlyoutSectionLabel>Input</FlyoutSectionLabel>
          <div className="mt-1 flex flex-col gap-0.5">
            {data!.sources.map((s) => (
              <FlyoutRadioRow
                key={s.id}
                label={s.name}
                checked={s.is_default}
                disabled={busy}
                onSelect={() => setDefaultMut.mutate({ device_id: s.id, type: "input" })}
              />
            ))}
          </div>
        </div>
      ) : null}

      <FlyoutExpandLink
        label="Audio settings"
        onClick={() => api.openControlCenterPane("audio")}
      />
    </FlyoutShell>
  )
}
