/** Bar battery icon color — white by default, red when low (no green/yellow tiers). */
export function batteryLevelColorClass(percent: number, charging?: boolean): string {
  if (!charging && percent <= 20) return "text-red"
  return "text-subtext1"
}

/** Material icon fallback where a custom SVG is not used. */
export function batteryMaterialIcon(percent: number, charging: boolean): string {
  const p = Math.max(0, Math.min(100, Math.round(percent)))
  if (charging) {
    if (p >= 95) return "battery_charging_full"
    if (p >= 80) return "battery_charging_80"
    if (p >= 60) return "battery_charging_60"
    if (p >= 50) return "battery_charging_50"
    if (p >= 30) return "battery_charging_30"
    return "battery_charging_20"
  }
  if (p >= 100) return "battery_full"
  const bar = Math.min(6, Math.floor((p / 100) * 7))
  return `battery_${bar}_bar`
}
