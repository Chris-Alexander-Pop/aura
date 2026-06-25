import { useState, useEffect } from "react"
import { motion, AnimatePresence } from "framer-motion"
import { cn } from "@/lib/utils"
import { postPanelHover } from "@/lib/panel-hover"
import {
  consumeControlCenterPane,
  subscribeControlCenterPane,
} from "@/lib/control-center-pane"
import { NAV_SECTIONS, type PaneId } from "./control-center/navigation"
import NetworkPane from "./control-center/panes/NetworkPane"
import WeatherPane from "./control-center/panes/WeatherPane"
import BluetoothPane from "./control-center/panes/BluetoothPane"
import FitnessPane from "./control-center/panes/FitnessPane"
import { PackagesPane } from "./control-center/panes/PackagesPane"
import { SettingsPane } from "./control-center/panes/SettingsPane"
import { VpnPane } from "./control-center/panes/VpnPane"
import { AudioPane } from "./control-center/panes/AudioPane"
import { NotificationsPane } from "./control-center/panes/NotificationsPane"
import { PerformancePane } from "./control-center/panes/PerformancePane"
import { KeybindsPane } from "./control-center/panes/KeybindsPane"
import { SecurityPane } from "./control-center/panes/SecurityPane"
import { ProductivityPane } from "./control-center/panes/ProductivityPane"
import { AutomationsPane } from "./control-center/panes/AutomationsPane"
import { CalendarNavPane } from "./control-center/panes/CalendarNavPane"
import { LogsPane } from "./control-center/panes/LogsPane"
import { DevopsPane } from "./control-center/panes/DevopsPane"
import { VaultPane } from "./control-center/panes/VaultPane"
import { CommunicationPane } from "./control-center/panes/CommunicationPane"

function renderPane(active: PaneId) {
  switch (active) {
    case "packages":
      return <PackagesPane />
    case "settings":
      return <SettingsPane />
    case "network":
      return <NetworkPane />
    case "vpn":
      return <VpnPane />
    case "bluetooth":
      return <BluetoothPane />
    case "audio":
      return <AudioPane />
    case "notifications":
      return <NotificationsPane />
    case "performance":
      return <PerformancePane />
    case "keybinds":
      return <KeybindsPane />
    case "security":
      return <SecurityPane />
    case "productivity":
      return <ProductivityPane />
    case "automations":
      return <AutomationsPane />
    case "calendar":
      return <CalendarNavPane />
    case "logs":
      return <LogsPane />
    case "devops":
      return <DevopsPane />
    case "vault":
      return <VaultPane />
    case "communication":
      return <CommunicationPane />
    case "fitness":
      return <FitnessPane />
    case "weather":
      return <WeatherPane />
    default: {
      const _exhaustive: never = active
      return _exhaustive
    }
  }
}

export default function ControlCenter() {
  const [active, setActive] = useState<PaneId>("network")
  const [navExpanded, setNavExpanded] = useState(false)

  useEffect(() => {
    const pending = consumeControlCenterPane()
    if (pending) setActive(pending)
    return subscribeControlCenterPane(setActive)
  }, [])

  return (
    <div
      className="flex h-full min-h-0 bg-base/80 backdrop-blur-2xl rounded-2xl overflow-hidden border border-surface0/60 shadow-2xl text-text"
      onMouseEnter={() => postPanelHover("controlCenterHover", true)}
      onMouseLeave={() => postPanelHover("controlCenterHover", false)}
    >
      <motion.nav
        animate={{ width: navExpanded ? 212 : 60 }}
        transition={{ type: "spring", stiffness: 400, damping: 35 }}
        className="flex flex-col pt-3 pb-3 bg-mantle/90 border-r border-surface0/60 overflow-hidden shrink-0"
      >
        <button
          onClick={() => setNavExpanded(!navExpanded)}
          className="icon-btn mx-auto mb-2"
          type="button"
          title="Toggle nav"
        >
          <span className="icon text-xl">{navExpanded ? "menu_open" : "menu"}</span>
        </button>

        <div className="flex flex-col gap-0.5 px-2 overflow-y-auto flex-1 min-h-0">
          {NAV_SECTIONS.map((section, si) => (
            <div
              key={section.label}
              className={cn("flex flex-col gap-0.5", si > 0 && "pt-2 mt-1 border-t border-surface0/40")}
            >
              {navExpanded && (
                <p className="text-[10px] uppercase tracking-wider text-subtext1/80 px-1.5 py-1.5">
                  {section.label}
                </p>
              )}
              {section.items.map((item) => (
                <button
                  key={item.id}
                  type="button"
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
          ))}
        </div>
      </motion.nav>

      <div className="flex-1 overflow-hidden relative min-w-0 min-h-0">
        {/* Plain keyed wrapper; min-h-0 so nested overflow-y-auto panes scroll inside flex layout */}
        <div key={active} className="absolute inset-0 min-h-0 overflow-hidden">
          {renderPane(active)}
        </div>
      </div>
    </div>
  )
}
