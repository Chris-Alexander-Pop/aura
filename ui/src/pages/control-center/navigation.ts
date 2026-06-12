/**
 * Control Center nav: order and groupings follow docs/roadmap/control-panel.md
 */
export const NAV_SECTIONS = [
  {
    label: "System",
    items: [
      { id: "packages", icon: "package_2", label: "Packages" },
      { id: "settings", icon: "settings", label: "System settings" },
    ],
  },
  {
    label: "Connectivity",
    items: [
      { id: "network", icon: "wifi", label: "Network" },
      { id: "vpn", icon: "vpn_key", label: "VPN" },
      { id: "bluetooth", icon: "bluetooth", label: "Bluetooth" },
      { id: "audio", icon: "tune", label: "Audio" },
      { id: "notifications", icon: "notifications", label: "Notifications" },
    ],
  },
  {
    label: "Tuning",
    items: [
      { id: "performance", icon: "speed", label: "Performance" },
      { id: "keybinds", icon: "keyboard", label: "Keybinds" },
      { id: "security", icon: "security", label: "Security" },
    ],
  },
  {
    label: "Habits & data",
    items: [
      { id: "productivity", icon: "task_alt", label: "Productivity" },
      { id: "automations", icon: "smart_toy", label: "Automations" },
      { id: "calendar", icon: "calendar_month", label: "Calendar" },
      { id: "vault", icon: "folder_special", label: "Vault" },
      { id: "logs", icon: "receipt_long", label: "Logs" },
    ],
  },
  {
    label: "Workloads",
    items: [
      { id: "devops", icon: "deployed_code", label: "DevOps" },
      { id: "communication", icon: "chat_bubble", label: "Communication" },
      { id: "fitness", icon: "fitness_center", label: "Fitness" },
    ],
  },
  {
    label: "Extras",
    items: [
      { id: "weather", icon: "partly_cloudy_day", label: "Weather" },
    ],
  },
] as const

export type PaneId = (typeof NAV_SECTIONS)[number]["items"][number]["id"]

export const ALL_NAV_ITEMS = NAV_SECTIONS.flatMap((section) => [...section.items])

export function getNavItem(
  id: PaneId
): (typeof ALL_NAV_ITEMS)[number] {
  const found = ALL_NAV_ITEMS.find((i) => i.id === id)
  if (!found) throw new Error(`Unknown pane id: ${id}`)
  return found
}
