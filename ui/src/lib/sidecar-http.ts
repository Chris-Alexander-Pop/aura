/** Shared HTTP auth for the sidecar REST API at localhost:9080. */

const META_URL = "/api/meta"
const SIDECAR_TIMEOUT_MS = 12_000

let cachedToken: string | null = null
let inflight: Promise<string> | null = null

export const SIDECAR_TOKEN_HEADER = "X-Aura-Token"

async function fetchHttpToken(): Promise<string> {
  const res = await fetch(META_URL, {
    method: "GET",
    signal: AbortSignal.timeout(SIDECAR_TIMEOUT_MS),
  })
  const text = await res.text()
  let json: { http_token?: unknown }
  try {
    json = JSON.parse(text) as { http_token?: unknown }
  } catch {
    throw new Error("Could not parse sidecar /api/meta — is ags-sidecar running?")
  }
  const token = json.http_token
  if (typeof token !== "string" || token.length < 16) {
    throw new Error("Sidecar /api/meta did not return http_token")
  }
  cachedToken = token
  return token
}

export async function sidecarHttpToken(): Promise<string> {
  if (cachedToken) return cachedToken
  if (!inflight) {
    inflight = fetchHttpToken().finally(() => {
      inflight = null
    })
  }
  return inflight
}

export async function sidecarAuthHeaders(): Promise<Record<string, string>> {
  const token = await sidecarHttpToken()
  return {
    "Content-Type": "application/json",
    [SIDECAR_TOKEN_HEADER]: token,
  }
}
