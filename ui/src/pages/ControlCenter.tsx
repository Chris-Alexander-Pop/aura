import { useState, useEffect } from "react"
import { motion, AnimatePresence } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { connectWs } from "@/lib/ws"
import { cn } from "@/lib/utils"

// ── Nav items ────────────────────────────────────────────────────────────────
const NAV_ITEMS = [
  { id: "network",       icon: "wifi",             label: "Network" },
  { id: "bluetooth",     icon: "bluetooth",         label: "Bluetooth" },
  { id: "audio",         icon: "tune",              label: "Audio" },
  { id: "notifications", icon: "notifications",     label: "Notifs" },
  { id: "keybinds",      icon: "keyboard",          label: "Keybinds" },
  { id: "performance",   icon: "speed",             label: "Performance" },
  { id: "security",      icon: "security",          label: "Security" },
  { id: "devops",        icon: "deployed_code",     label: "DevOps" },
  { id: "productivity",  icon: "task_alt",          label: "Productivity" },
  { id: "calendar",      icon: "calendar_month",    label: "Calendar" },
  { id: "logs",          icon: "receipt_long",      label: "Logs" },
  { id: "packages",      icon: "package_2",         label: "Packages" },
  { id: "automation",    icon: "smart_toy",         label: "Automation" },
  { id: "communication", icon: "chat_bubble",       label: "Messages" },
  { id: "fitness",       icon: "fitness_center",    label: "Fitness" },
  { id: "weather",       icon: "partly_cloudy_day", label: "Weather" },
] as const

type PaneId = (typeof NAV_ITEMS)[number]["id"]

// ── Stub pane ─────────────────────────────────────────────────────────────────
function StubPane({ label, icon }: { label: string; icon: string }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.18 }}
      className="flex flex-col gap-6 p-6 h-full"
    >
      <div>
        <div className="flex items-center gap-3 mb-1">
          <span className="icon text-mauve text-2xl">{icon}</span>
          <h2 className="text-xl font-semibold text-text">{label}</h2>
        </div>
        <p className="text-sm text-subtext0">Stub — implementation coming soon</p>
      </div>
      {/* Skeleton cards */}
      {[...Array(4)].map((_, i) => (
        <div key={i} className="skeleton h-16 rounded-xl" style={{ opacity: 1 - i * 0.15 }} />
      ))}
    </motion.div>
  )
}

// ── Network pane ──────────────────────────────────────────────────────────────
function NetworkPane() {
  const { data: status, isLoading } = useQuery({
    queryKey: ["network-status"],
    queryFn: api.getNetworkStatus,
    refetchInterval: 5000,
  })
  const { data: aps, refetch: scan } = useQuery({
    queryKey: ["network-scan"],
    queryFn: api.scanNetworks,
    enabled: false,
  })

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0 }}
      className="flex flex-col gap-4 p-6 h-full overflow-y-auto"
    >
      <div className="flex items-center gap-3">
        <span className="icon text-mauve text-2xl">wifi</span>
        <h2 className="text-xl font-semibold">Network</h2>
      </div>

      {/* Status card */}
      <div className="glass-card p-4 flex flex-col gap-3">
        {isLoading ? (
          <div className="skeleton h-12 rounded-lg" />
        ) : (
          <>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-text">
                  {status?.active_connection ?? "Not connected"}
                </p>
                <p className="text-xs text-subtext0">{status?.local_ip ?? "—"}</p>
              </div>
              <button
                onClick={() => api.toggleWifi(!status?.wifi_enabled)}
                className={cn("toggle-chip text-xs", status?.wifi_enabled && "active")}
              >
                <span className="icon text-base">wifi</span>
                {status?.wifi_enabled ? "On" : "Off"}
              </button>
            </div>
            <p className="text-xs text-subtext0">Public: {status?.public_ip ?? "—"}</p>
          </>
        )}
      </div>

      {/* Scan */}
      <button className="btn-surface text-sm" onClick={() => scan()}>
        <span className="icon text-base">radar</span>
        Scan networks
      </button>

      {aps && aps.length > 0 && (
        <div className="flex flex-col gap-2">
          {aps.map((ap) => (
            <div
              key={ap.ssid}
              className={cn("glass-card p-3 flex items-center gap-3 cursor-pointer hover:bg-surface1/50 transition-colors", ap.active && "border-mauve/40")}
            >
              <span className="icon text-base text-subtext1">
                {ap.strength > 70 ? "signal_wifi_4_bar" : ap.strength > 40 ? "network_wifi_3_bar" : "signal_wifi_1_bar"}
              </span>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium truncate">{ap.ssid}</p>
                <p className="text-xs text-subtext0">{ap.security} · {ap.strength}%</p>
              </div>
              {ap.active && <span className="icon text-blue text-sm">check_circle</span>}
            </div>
          ))}
        </div>
      )}
    </motion.div>
  )
}

// ── Weather pane ──────────────────────────────────────────────────────────────
function WeatherPane() {
  const { data, isLoading } = useQuery({
    queryKey: ["weather"],
    queryFn: api.getWeather,
    refetchInterval: 300_000,
  })

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0 }}
      className="flex flex-col gap-4 p-6"
    >
      <div className="flex items-center gap-3">
        <span className="icon text-mauve text-2xl">partly_cloudy_day</span>
        <h2 className="text-xl font-semibold">Weather</h2>
      </div>
      {isLoading ? (
        <div className="skeleton h-28 rounded-xl" />
      ) : data ? (
        <div className="glass-card p-5 flex flex-col gap-2">
          <div className="flex items-end gap-4">
            <span className="text-5xl font-bold text-text">{data.temp}</span>
            <span className="text-subtext0 text-sm mb-2">Feels like {data.feels_like}</span>
          </div>
          <p className="text-subtext1 capitalize">{data.description}</p>
          <p className="text-xs text-subtext0">Humidity: {data.humidity}%</p>
        </div>
      ) : (
        <p className="text-subtext0 text-sm">No weather data — configure location in sidecar</p>
      )}
    </motion.div>
  )
}

// ── Main ──────────────────────────────────────────────────────────────────────
export default function ControlCenter() {
  const [active, setActive] = useState<PaneId>("network")
  const [navExpanded, setNavExpanded] = useState(false)

  useEffect(() => {
    connectWs()
  }, [])

  const renderPane = () => {
    switch (active) {
      case "network":      return <NetworkPane />
      case "weather":      return <WeatherPane />
      default: {
        const item = NAV_ITEMS.find((n) => n.id === active)!
        return <StubPane label={item.label} icon={item.icon} />
      }
    }
  }

  return (
    <div className="flex h-full bg-base/80 backdrop-blur-2xl rounded-2xl overflow-hidden border border-surface0/60 shadow-2xl text-text">
      {/* Nav Rail */}
      <motion.nav
        animate={{ width: navExpanded ? 200 : 60 }}
        transition={{ type: "spring", stiffness: 400, damping: 35 }}
        className="flex flex-col pt-3 pb-3 bg-mantle/90 border-r border-surface0/60 overflow-hidden shrink-0"
      >
        {/* Expand toggle */}
        <button
          onClick={() => setNavExpanded(!navExpanded)}
          className="icon-btn mx-auto mb-3"
          title="Toggle nav"
        >
          <span className="icon text-xl">{navExpanded ? "menu_open" : "menu"}</span>
        </button>

        <div className="flex flex-col gap-0.5 px-2 overflow-y-auto flex-1">
          {NAV_ITEMS.map((item) => (
            <button
              key={item.id}
              onClick={() => setActive(item.id)}
              className={cn("nav-item w-full justify-start", active === item.id && "active")}
              title={item.label}
            >
              <span className="icon text-xl shrink-0">{item.icon}</span>
              <AnimatePresence>
                {navExpanded && (
                  <motion.span
                    initial={{ opacity: 0, width: 0 }}
                    animate={{ opacity: 1, width: "auto" }}
                    exit={{ opacity: 0, width: 0 }}
                    className="text-xs overflow-hidden whitespace-nowrap"
                  >
                    {item.label}
                  </motion.span>
                )}
              </AnimatePresence>
            </button>
          ))}
        </div>
      </motion.nav>

      {/* Pane area */}
      <div className="flex-1 overflow-hidden relative">
        <AnimatePresence mode="wait">
          <motion.div key={active} className="absolute inset-0">
            {renderPane()}
          </motion.div>
        </AnimatePresence>
      </div>
    </div>
  )
}
