import { useEffect } from "react"
import { motion } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { connectWs } from "@/lib/ws"

const container = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.06, delayChildren: 0.05 },
  },
}

const item = {
  hidden: { opacity: 0, y: 16 },
  show: { opacity: 1, y: 0, transition: { type: "spring", stiffness: 380, damping: 28 } },
}

// ── Tile: Wifi / BT / Battery row ────────────────────────────────────────────
function QuickTileRow() {
  const { data: net } = useQuery({ queryKey: ["network-status"], queryFn: api.getNetworkStatus, refetchInterval: 8000 })
  const { data: batt } = useQuery({ queryKey: ["battery"], queryFn: api.getBatteryState, refetchInterval: 10000 })

  return (
    <motion.div variants={item} className="glass-card p-4 flex gap-3">
      {/* Wifi */}
      <button
        onClick={() => api.toggleWifi(!net?.wifi_enabled)}
        className={`toggle-chip flex-1 flex-col gap-1 py-3 ${net?.wifi_enabled ? "active" : ""}`}
      >
        <span className="icon text-2xl">wifi</span>
        <span className="text-xs">{net?.wifi_enabled ? net.active_connection ?? "Wi-Fi" : "Off"}</span>
      </button>

      {/* Bluetooth stub */}
      <button className="toggle-chip flex-1 flex-col gap-1 py-3">
        <span className="icon text-2xl">bluetooth</span>
        <span className="text-xs">Bluetooth</span>
      </button>

      {/* Battery */}
      <div className="toggle-chip flex-1 flex-col gap-1 py-3 cursor-default">
        <span className="icon text-2xl">
          {batt
            ? batt.charging
              ? "battery_charging_full"
              : batt.percent > 20
              ? "battery_5_bar"
              : "battery_1_bar"
            : "battery_unknown"}
        </span>
        <span className="text-xs">{batt ? `${batt.percent}%` : "—"}</span>
      </div>
    </motion.div>
  )
}

// ── Tile: Clock + Date ────────────────────────────────────────────────────────
function ClockTile() {
  const now = new Date()
  const time = now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
  const date = now.toLocaleDateString([], { weekday: "long", month: "long", day: "numeric" })

  return (
    <motion.div variants={item} className="glass-card p-5 text-center">
      <p className="text-4xl font-bold tracking-tight text-text">{time}</p>
      <p className="text-sm text-subtext0 mt-1">{date}</p>
    </motion.div>
  )
}

// ── Tile: Power actions ───────────────────────────────────────────────────────
function PowerTile() {
  const actions = [
    { icon: "lock", label: "Lock",     cmd: "loginctl lock-session" },
    { icon: "logout", label: "Logout", cmd: "hyprctl dispatch exit" },
    { icon: "power_settings_new", label: "Shutdown", cmd: "systemctl poweroff" },
    { icon: "restart_alt", label: "Restart",  cmd: "systemctl reboot" },
  ]

  return (
    <motion.div variants={item} className="glass-card p-4">
      <p className="text-xs text-subtext0 font-medium mb-3 uppercase tracking-wider">Session</p>
      <div className="grid grid-cols-4 gap-2">
        {actions.map((a) => (
          <button key={a.label} className="icon-btn flex-col gap-1 h-14 rounded-xl hover:bg-red/10 hover:text-red" title={a.label}>
            <span className="icon text-xl">{a.icon}</span>
            <span className="text-[10px]">{a.label}</span>
          </button>
        ))}
      </div>
    </motion.div>
  )
}

// ── Tile: App launcher strip ──────────────────────────────────────────────────
const APP_TILES = [
  { icon: "terminal",    label: "Terminal",  cmd: "kitty" },
  { icon: "globe",       label: "Browser",   cmd: "firefox" },
  { icon: "folder",      label: "Files",     cmd: "nautilus" },
  { icon: "code",        label: "VSCode",    cmd: "code" },
  { icon: "music_note",  label: "Spotify",   cmd: "spotify" },
  { icon: "message",     label: "Discord",   cmd: "discord" },
]

function AppTile() {
  return (
    <motion.div variants={item} className="glass-card p-4">
      <p className="text-xs text-subtext0 font-medium mb-3 uppercase tracking-wider">Launch</p>
      <div className="grid grid-cols-6 gap-2">
        {APP_TILES.map((app) => (
          <button
            key={app.label}
            className="icon-btn flex-col gap-1 h-14 w-full rounded-xl"
            title={app.label}
          >
            <span className="icon text-xl">{app.icon}</span>
            <span className="text-[10px] text-subtext0">{app.label}</span>
          </button>
        ))}
      </div>
    </motion.div>
  )
}

// ── Main ──────────────────────────────────────────────────────────────────────
export default function Sidebar() {
  useEffect(() => { connectWs() }, [])

  return (
    <div className="flex flex-col h-full bg-base/80 backdrop-blur-2xl rounded-2xl border border-surface0/60 shadow-2xl overflow-hidden p-3 gap-3">
      <motion.div
        variants={container}
        initial="hidden"
        animate="show"
        className="flex flex-col gap-3 flex-1"
      >
        <ClockTile />
        <QuickTileRow />
        <AppTile />
        <div className="flex-1" />
        <PowerTile />
      </motion.div>
    </div>
  )
}
