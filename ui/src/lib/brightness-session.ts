/** Single-flight brightness RPC coalescing + WS stale-event guard. */

const LIVE_DEBOUNCE_MS = 120

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
  let inFlight = false
  let pending: number | null = null
  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  const clearDebounce = () => {
    if (debounceTimer != null) {
      clearTimeout(debounceTimer)
      debounceTimer = null
    }
  }

  const pump = () => {
    if (inFlight || pending == null) return
    const next = pending
    pending = null
    inFlight = true
    void send(next)
      .then((result) => {
        onSuccess?.(result)
      })
      .catch((err) => {
        onError?.(err)
      })
      .finally(() => {
        inFlight = false
        pump()
      })
  }

  const schedule = (frac: number) => {
    pending = frac
    clearDebounce()
    debounceTimer = setTimeout(() => {
      debounceTimer = null
      pump()
    }, LIVE_DEBOUNCE_MS)
  }

  const flush = (frac: number) => {
    pending = frac
    clearDebounce()
    pump()
  }

  return {
    setTarget(frac, opts) {
      if (opts?.flush) flush(frac)
      else schedule(frac)
    },
    isBusy() {
      return inFlight || pending != null || debounceTimer != null
    },
    shouldApplyWsEvent() {
      return !this.isBusy()
    },
    dispose() {
      clearDebounce()
      pending = null
    },
  }
}
