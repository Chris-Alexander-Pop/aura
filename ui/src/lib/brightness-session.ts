/** Single-flight brightness RPC coalescing + WS stale-event guard. */

const LIVE_DEBOUNCE_MS = 50

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

  const pump = (waitForResponse: boolean) => {
    if (inFlight || pending == null) return
    const next = pending
    pending = null
    inFlight = true
    const task = send(next)
    if (waitForResponse) {
      void task
        .then((result) => {
          onSuccess?.(result)
        })
        .catch((err) => {
          onError?.(err)
        })
        .finally(() => {
          inFlight = false
          pump(false)
        })
    } else {
      void task
        .catch((err) => {
          onError?.(err)
        })
        .finally(() => {
          inFlight = false
          pump(false)
        })
    }
  }

  const schedule = (frac: number) => {
    pending = frac
    clearDebounce()
    debounceTimer = setTimeout(() => {
      debounceTimer = null
      pump(false)
    }, LIVE_DEBOUNCE_MS)
  }

  const flush = (frac: number) => {
    pending = frac
    clearDebounce()
    pump(true)
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
