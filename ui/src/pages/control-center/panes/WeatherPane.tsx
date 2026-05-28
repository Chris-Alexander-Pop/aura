import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { cn } from "@/lib/utils"

const WEATHER_QUERY_KEY = ["weather"] as const

/** Sidecar `Weather.Get` icon tokens → Material Symbols Rounded names */
const WEATHER_MATERIAL_ICON: Record<string, string> = {
  sunny: "wb_sunny",
  partly_cloudy: "partly_cloudy_day",
  cloudy: "cloud",
  foggy: "foggy",
  rainy: "rainy",
  snowy: "snowing",
  cloud_alert: "thunderstorm",
}

function materialWeatherIcon(sidecarIcon: string): string {
  return WEATHER_MATERIAL_ICON[sidecarIcon] ?? "partly_cloudy_day"
}

/** 5 minute ambient refresh — matches previous ControlCenter inline pane */
const REFETCH_MS = 300_000

export default function WeatherPane() {
  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: WEATHER_QUERY_KEY,
    queryFn: api.getWeather,
    staleTime: 120_000,
    refetchInterval: REFETCH_MS,
    retry: 2,
  })

  const errMsg =
    error instanceof Error ? error.message : error != null ? String(error) : "Could not load weather."

  const headerIcon =
    data != null ? materialWeatherIcon(data.icon) : WEATHER_MATERIAL_ICON.partly_cloudy

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0 }}
      className="flex flex-col gap-4 p-6 h-full overflow-y-auto"
    >
      <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <div className="flex items-center gap-3">
            <span className={cn("icon text-mauve text-2xl", isFetching && data && "opacity-60")}>
              {headerIcon}
            </span>
            <h2 className="text-xl font-semibold">Weather</h2>
          </div>
          <p className="text-xs text-subtext1 mt-1 max-w-prose">
            Ambient quick readout (sidecar wttr/in). Configure location via the Aura sidecar weather
            settings.
          </p>
        </div>
        <button
          type="button"
          className="btn-surface text-sm shrink-0 self-start"
          disabled={isFetching}
          onClick={() => void refetch()}
        >
          <span className="icon text-base">refresh</span>
          {isFetching ? "Refreshing…" : "Refresh"}
        </button>
      </div>

      {isLoading ? (
        <div className="skeleton h-28 rounded-xl" />
      ) : isError ? (
        <div className="glass-card p-5 flex flex-col gap-3">
          <p className="text-sm text-subtext1">{errMsg}</p>
          <p className="text-xs text-subtext0">
            Check that ags-sidecar is running and network access to the weather source is available.
          </p>
          <button type="button" className="btn-surface text-sm self-start" onClick={() => void refetch()}>
            Try again
          </button>
        </div>
      ) : data ? (
        <div className="glass-card p-5 flex flex-col gap-3">
          <div className="flex items-end gap-4 flex-wrap">
            <span className="icon text-6xl text-mauve/90 leading-none select-none">
              {materialWeatherIcon(data.icon)}
            </span>
            <div className="flex flex-col gap-0.5 min-w-0">
              <div className="flex items-end gap-3 flex-wrap">
                <span className="text-5xl font-bold text-text tabular-nums">{data.temp}</span>
                <span className="text-subtext0 text-sm mb-2">Feels like {data.feels_like}</span>
              </div>
              <p className="text-subtext1 capitalize">{data.description}</p>
            </div>
          </div>
          <p className="text-xs text-subtext0">Humidity: {data.humidity}%</p>
        </div>
      ) : (
        <p className="text-subtext0 text-sm">No weather data — configure location in the sidecar.</p>
      )}
    </motion.div>
  )
}
