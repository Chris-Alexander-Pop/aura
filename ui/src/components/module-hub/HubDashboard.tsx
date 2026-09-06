import { useEffect, useState } from "react"
import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import type { DashboardQuickStatusView } from "@/lib/api-types"
import { cn, tempToBarPercent, usageToPercent } from "@/lib/utils"

const WEATHER_REFETCH_MS = 300_000

function toEpochMs(t: number): number {
  if (!Number.isFinite(t)) return NaN
  return Math.abs(t) > 1e12 ? t : t * 1000
}

function formatEventTime(ev: { start: number; end: number }): string {
  const s = toEpochMs(ev.start)
  if (!Number.isFinite(s)) return ""
  return new Date(s).toLocaleTimeString([], { hour: "numeric", minute: "2-digit" })
}

function ResourceBars() {
  const { data, isLoading, isError } = useQuery({
    queryKey: ["system-stats"],
    queryFn: api.getSystemStats,
    refetchInterval: 2000,
    refetchIntervalInBackground: true,
    staleTime: 0,
  })

  if (isLoading && !data) {
    return <p className="text-[11px] text-subtext0">Loading resources…</p>
  }
  if (isError && !data) {
    return <p className="text-[11px] text-red">Resources unavailable</p>
  }

  const cpu = usageToPercent(data?.cpu ?? 0)
  const ram = usageToPercent(data?.ram ?? 0)
  const tempC = data?.temp ?? 0
  const stats = [
    { label: "CPU", value: cpu, text: `${cpu.toFixed(0)}%`, color: "from-blue to-sapphire" },
    { label: "RAM", value: ram, text: `${ram.toFixed(0)}%`, color: "from-mauve to-pink" },
    {
      label: "Temp",
      value: tempToBarPercent(tempC),
      text: Number.isFinite(tempC) && tempC > 0 ? `${tempC.toFixed(0)}°` : "—",
      color: "from-peach to-maroon",
    },
  ]

  return (
    <div className="flex gap-3">
      {stats.map((s) => (
        <div key={s.label} className="flex min-w-0 flex-1 flex-col gap-1">
          <div className="flex justify-between text-[11px] text-subtext0">
            <span>{s.label}</span>
            <span>{s.text}</span>
          </div>
          <div className="progress-bar">
            <motion.div
              className={cn("progress-fill bg-gradient-to-r", s.color)}
              initial={{ width: 0 }}
              animate={{ width: `${s.value}%` }}
              transition={{ duration: 0.5 }}
            />
          </div>
        </div>
      ))}
    </div>
  )
}

export function HubDashboard({ quick }: { quick: DashboardQuickStatusView | undefined }) {
  const [now, setNow] = useState(() => new Date())
  useEffect(() => {
    const id = window.setInterval(() => setNow(new Date()), 1000)
    return () => window.clearInterval(id)
  }, [])

  const { data: weather, isLoading: weatherLoading, isError: weatherError } = useQuery({
    queryKey: ["weather"],
    queryFn: api.getWeather,
    staleTime: 120_000,
    refetchInterval: WEATHER_REFETCH_MS,
    retry: 1,
  })

  const timeStr = now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
  const dateStr = now.toLocaleDateString([], { weekday: "short", month: "short", day: "numeric" })

  const weatherLine = weatherLoading
    ? "Weather…"
    : weatherError || !weather
      ? "Weather unavailable"
      : `${weather.temp} · ${weather.description}`

  const next = quick?.next_event

  return (
    <div className="flex flex-col gap-2">
      <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
        <button
          type="button"
          className="flex flex-col rounded-xl bg-surface0/35 p-3 text-left transition-colors hover:bg-surface0/55"
          title="Open calendar"
          onClick={() => void api.auraToggleWindow("calendar")}
        >
          <span className="text-2xl font-semibold tabular-nums leading-none">{timeStr}</span>
          <span className="mt-1 text-[11px] text-subtext0">{dateStr}</span>
        </button>

        <div className="flex flex-col justify-center rounded-xl bg-surface0/35 p-3">
          <span className="text-[10px] font-semibold uppercase tracking-wide text-subtext1">
            Weather
          </span>
          <p className="mt-1 line-clamp-2 text-sm text-text" title={weatherLine}>
            {weatherLine}
          </p>
        </div>

        <button
          type="button"
          className="flex flex-col rounded-xl bg-surface0/35 p-3 text-left transition-colors hover:bg-surface0/55"
          title="Open calendar"
          onClick={() => void api.auraToggleWindow("calendar")}
        >
          <span className="text-[10px] font-semibold uppercase tracking-wide text-subtext1">
            Next event
          </span>
          {next?.title ? (
            <>
              <span className="mt-1 line-clamp-1 text-sm font-medium text-text">{next.title}</span>
              <span className="text-[11px] text-subtext0">{formatEventTime(next)}</span>
            </>
          ) : (
            <span className="mt-1 text-sm text-subtext0">Nothing upcoming</span>
          )}
        </button>
      </div>

      <div className="rounded-xl bg-surface0/35 px-3 py-2">
        <ResourceBars />
      </div>
    </div>
  )
}
