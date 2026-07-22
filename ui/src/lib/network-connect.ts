/** Shared Wi‑Fi connect helpers — Caelestia-style (try saved profile first). */

export class NetworkConnectError extends Error {
  readonly needsPassword: boolean

  constructor(message: string, needsPassword = false) {
    super(message)
    this.name = "NetworkConnectError"
    this.needsPassword = needsPassword
  }
}

export function isSecuredNetwork(security: string): boolean {
  const s = security.trim().toLowerCase()
  return s.length > 0 && s !== "none" && s !== "open" && s !== "--"
}

export function strengthIcon(strength: number): string {
  if (strength >= 75) return "wifi"
  if (strength >= 50) return "wifi_2_bar"
  if (strength >= 25) return "wifi_1_bar"
  return "wifi_1_bar"
}

export function isPasswordRequiredError(err: unknown): boolean {
  if (err instanceof NetworkConnectError) return err.needsPassword
  if (!(err instanceof Error)) return false
  const m = err.message.toLowerCase()
  return m.includes("password required") || m.includes("incorrect wi")
}

/** Desktop id for Remmina (org.remmina.Remmina.desktop). */
export const REMMINA_DESKTOP_ID = "org.remmina.Remmina"
