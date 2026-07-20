/** Best-effort frontend crash reporting → sidecar `Crash.Report` → local dump files. */

const BASE = "/api"

export type FrontendCrashKind = "uncaught" | "exit" | "web-process"

export interface FrontendCrashPayload {
  message: string
  kind?: FrontendCrashKind
  stack?: string
  route?: string
}

let reporting = false
const recent = new Set<string>()

function routeHint(): string {
  try {
    return window.location.hash || window.location.pathname || ""
  } catch {
    return ""
  }
}

function dedupeKey(message: string, stack?: string): string {
  return `${message}\n${(stack ?? "").slice(0, 200)}`
}

/** Fire-and-forget; never throws. Dedupes identical dumps for 30s. */
export function reportFrontendCrash(payload: FrontendCrashPayload): void {
  const message = (payload.message || "unknown error").slice(0, 2048)
  const stack = payload.stack?.slice(0, 16_384)
  const key = dedupeKey(message, stack)
  if (recent.has(key)) return
  recent.add(key)
  setTimeout(() => recent.delete(key), 30_000)

  const body = {
    message,
    kind: payload.kind ?? "uncaught",
    stack,
    route: payload.route ?? routeHint(),
  }

  try {
    void fetch(`${BASE}/${encodeURIComponent("Crash.Report")}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      keepalive: true,
      signal: AbortSignal.timeout(5_000),
    }).catch(() => {
      /* sidecar down — nothing to do */
    })
  } catch {
    /* ignore */
  }
}

export function installFrontendCrashHandlers(): void {
  if (reporting) return
  reporting = true

  window.addEventListener("error", (event) => {
    const err = event.error
    reportFrontendCrash({
      message: err instanceof Error ? err.message : event.message || "window.error",
      stack: err instanceof Error ? err.stack : undefined,
      kind: "uncaught",
    })
  })

  window.addEventListener("unhandledrejection", (event) => {
    const reason = event.reason
    const message =
      reason instanceof Error
        ? reason.message
        : typeof reason === "string"
          ? reason
          : "unhandledrejection"
    reportFrontendCrash({
      message,
      stack: reason instanceof Error ? reason.stack : undefined,
      kind: "uncaught",
    })
  })
}
