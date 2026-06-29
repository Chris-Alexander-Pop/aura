/** Throttled live brightness RPC + WS stale-event guard. */

const LIVE_THROTTLE_MS = 16
const WS_SUPPRESS_MS = 300
/** Match sidecar `BRIGHTNESS_MIN` — 0% turns some backlights off. */
const BRIGHTNESS_MIN_FRAC = 0.01

export type BrightnessSetResult = {
  brightness: number
  monitor?: string
}

export type BrightnessSession = {
  setTarget: (frac: number, opts?: { flush?: boolean }) => void
  isBusy: () => boolean
  shouldApplyWsEvent: () => boolean
  dispose: () => void
}

let activeSession: BrightnessSession | null = null

/** Register the flyout-owned session so WS handlers can suppress stale pushes. */
export function registerBrightnessSession(session: BrightnessSession | null) {
  activeSession = session
}

export function shouldApplyBrightnessWsEvent(): boolean {
  return activeSession?.shouldApplyWsEvent() ?? true
}

export function createBrightnessSession(
  send: (frac: number) => Promise<BrightnessSetResult>,
  onSuccess?: (result: BrightnessSetResult) => void,
  onError?: (err: unknown) => void,
): BrightnessSession {
  let lastLiveSendAt = -LIVE_THROTTLE_MS
  let pendingLive: number | null = null
  let throttleTimer: ReturnType<typeof setTimeout> | null = null
  let flushInFlight = false
  let suppressWsUntil = 0

  const touchWsSuppress = () => {
    suppressWsUntil = Date.now() + WS_SUPPRESS_MS
  }

  const clearThrottle = () => {
    if (throttleTimer != null) {
      clearTimeout(throttleTimer)
      throttleTimer = null
    }
  }

  const sendLive = (frac: number) => {
    lastLiveSendAt = Date.now()
    pendingLive = null
    void send(frac).catch((err) => {
      onError?.(err)
    })
  }

  const scheduleLive = (frac: number) => {
    pendingLive = frac
    touchWsSuppress()

    const elapsed = Date.now() - lastLiveSendAt
    if (elapsed >= LIVE_THROTTLE_MS) {
      clearThrottle()
      sendLive(frac)
      return
    }

    if (throttleTimer != null) return

    throttleTimer = setTimeout(() => {
      throttleTimer = null
      if (pendingLive != null) {
        sendLive(pendingLive)
      }
    }, LIVE_THROTTLE_MS - elapsed)
  }

  const flush = (frac: number) => {
    clearThrottle()
    pendingLive = null
    touchWsSuppress()

    flushInFlight = true
    void send(frac)
      .then((result) => {
        onSuccess?.(result)
      })
      .catch((err) => {
        onError?.(err)
      })
      .finally(() => {
        flushInFlight = false
        touchWsSuppress()
      })
  }

  return {
    setTarget(frac, opts) {
      const clamped = Math.min(1, Math.max(BRIGHTNESS_MIN_FRAC, frac))
      if (opts?.flush) flush(clamped)
      else scheduleLive(clamped)
    },
    isBusy() {
      return (
        flushInFlight ||
        throttleTimer != null ||
        pendingLive != null ||
        Date.now() < suppressWsUntil
      )
    },
    shouldApplyWsEvent() {
      return Date.now() >= suppressWsUntil
    },
    dispose() {
      clearThrottle()
      pendingLive = null
      suppressWsUntil = 0
    },
  }
}
