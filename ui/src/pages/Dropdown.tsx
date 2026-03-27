import { useEffect, useState } from "react"
import { motion, AnimatePresence } from "framer-motion"
import { useQuery } from "@tanstack/react-query"
import api from "@/lib/api"
import { connectWs } from "@/lib/ws"
import { cn } from "@/lib/utils"

// ── Quick toggle button ───────────────────────────────────────────────────────
function QuickToggle({
  icon,
  label,
  active,
  onClick,
}: {
  icon: string
  label: string
  active?: boolean
  onClick?: () => void
}) {
  return (
    <motion.button
      whileHover={{ scale: 1.04 }}
      whileTap={{ scale: 0.95 }}
      onClick={onClick}
      className={cn("toggle-chip flex-1 flex-col gap-1 py-3 text-center", active && "active")}
    >
      <span className="icon text-2xl">{icon}</span>
      <span className="text-[11px]">{label}</span>
    </motion.button>
  )
}

// ── System stats mini bar ─────────────────────────────────────────────────────
function MiniStats() {
  const { data } = useQuery({
    queryKey: ["system-stats"],
    queryFn: api.getSystemStats,
    refetchInterval: 4000,
  })

  const stats = [
    { label: "CPU", value: data?.cpu ?? 0, color: "from-blue to-sapphire" },
    { label: "RAM", value: data?.ram ?? 0, color: "from-mauve to-pink" },
    { label: "Temp", value: data?.temp ? data.temp / 100 : 0, color: "from-peach to-maroon" },
  ]

  return (
    <div className="flex gap-3">
      {stats.map((s) => (
        <div key={s.label} className="flex flex-col gap-1 flex-1">
          <div className="flex justify-between text-[11px] text-subtext0">
            <span>{s.label}</span>
            <span>
              {s.label === "Temp"
                ? `${data?.temp?.toFixed(0) ?? "—"}°`
                : `${s.value.toFixed(0)}%`}
            </span>
          </div>
          <div className="progress-bar">
            <motion.div
              className={`progress-fill bg-gradient-to-r ${s.color}`}
              initial={{ width: 0 }}
              animate={{ width: `${Math.min(s.value, 100)}%` }}
              transition={{ duration: 0.5 }}
            />
          </div>
        </div>
      ))}
    </div>
  )
}

export default function Dropdown() {
  const { data: net } = useQuery({ queryKey: ["network-status"], queryFn: api.getNetworkStatus, refetchInterval: 8000 })
  const { data: batt } = useQuery({ queryKey: ["battery"], queryFn: api.getBatteryState, refetchInterval: 10000 })
  const [dnd, setDnd] = useState(false)
  const [nightMode, setNightMode] = useState(false)

  useEffect(() => { connectWs() }, [])

  return (
    <motion.div
      initial={{ opacity: 0, y: -16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ type: "spring", stiffness: 400, damping: 30 }}
      className="flex flex-col gap-3 h-full bg-mantle/90 backdrop-blur-2xl border border-surface0/60 rounded-2xl shadow-2xl p-4 overflow-hidden"
    >
      {/* Top row: quick toggles */}
      <div className="flex gap-2">
        <QuickToggle
          icon="wifi"
          label={net?.active_connection ?? "Wi-Fi"}
          active={net?.wifi_enabled}
          onClick={() => api.toggleWifi(!net?.wifi_enabled)}
        />
        <QuickToggle icon="bluetooth" label="Bluetooth" />
        <QuickToggle
          icon="do_not_disturb_on"
          label="DnD"
          active={dnd}
          onClick={() => setDnd(!dnd)}
        />
        <QuickToggle
          icon="bedtime"
          label="Night"
          active={nightMode}
          onClick={() => setNightMode(!nightMode)}
        />
        <QuickToggle
          icon={batt?.charging ? "battery_charging_full" : "battery_5_bar"}
          label={batt ? `${batt.percent}%` : "Batt"}
        />
        <QuickToggle icon="screenshot_monitor" label="Screen" />
      </div>

      {/* Stats bar */}
      <div className="glass-card px-4 py-3">
        <MiniStats />
      </div>
    </motion.div>
  )
}
