/** Per-panel flyout widths — mirrors src/lib/theme.ts bar.popoutWidth */
export const FLYOUT_WIDTHS = {
  network: 340,
  bluetooth: 300,
  audio: 300,
  brightness: 260,
  battery: 250,
  windows: 300,
  power: 260,
} as const

export type FlyoutPanelId = keyof typeof FLYOUT_WIDTHS

export const FLYOUT_MAX_W = Math.max(...Object.values(FLYOUT_WIDTHS))

export function flyoutWidthFor(panel: string): number {
  if (panel in FLYOUT_WIDTHS) return FLYOUT_WIDTHS[panel as FlyoutPanelId]
  return FLYOUT_MAX_W
}
