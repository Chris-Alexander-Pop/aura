/** Material icon names for volume/brightness — matches OSD + StatusCluster tiers. */

export function volumeIcon(pct: number, muted: boolean): string {
  if (muted || pct <= 0) return "volume_off"
  if (pct <= 33) return "volume_mute"
  if (pct <= 66) return "volume_down"
  return "volume_up"
}

export function brightnessIcon(pct: number): string {
  if (pct <= 25) return "brightness_2"
  if (pct <= 50) return "brightness_4"
  if (pct <= 75) return "brightness_6"
  return "brightness_7"
}
