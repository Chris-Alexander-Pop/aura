import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import {
  createBrightnessSession,
  registerBrightnessSession,
  shouldApplyBrightnessWsEvent,
} from "./brightness-session"

describe("createBrightnessSession", () => {
  beforeEach(() => {
    vi.useFakeTimers()
    registerBrightnessSession(null)
  })

  afterEach(() => {
    vi.useRealTimers()
    registerBrightnessSession(null)
  })

  it("single_flight_coalesces rapid setTarget into one in-flight send", async () => {
    let inFlight = 0
    let resolveSend: ((v: { brightness: number }) => void) | undefined

    const session = createBrightnessSession(() => {
      inFlight += 1
      return new Promise<{ brightness: number }>((resolve) => {
        resolveSend = resolve
      })
    })

    session.setTarget(0.2)
    session.setTarget(0.4)
    session.setTarget(0.6)
    session.setTarget(0.8)

    expect(inFlight).toBe(0)
    expect(session.isBusy()).toBe(true)

    await vi.advanceTimersByTimeAsync(120)

    expect(inFlight).toBe(1)
    expect(session.isBusy()).toBe(true)
    resolveSend?.({ brightness: 0.8 })

    await vi.runAllTimersAsync()
    for (let i = 0; i < 5; i++) await Promise.resolve()

    expect(session.isBusy()).toBe(false)
  })

  it("flush_sends_immediately without waiting for debounce", async () => {
    const order: number[] = []
    const session = createBrightnessSession(async (frac) => {
      order.push(frac)
      return { brightness: frac }
    })

    session.setTarget(0.5, { flush: true })

    await Promise.resolve()

    expect(order).toEqual([0.5])
  })

  it("ws_suppressed_while_busy via registerBrightnessSession", async () => {
    const session = createBrightnessSession(() => new Promise(() => {}))
    registerBrightnessSession(session)

    session.setTarget(0.4)
    expect(shouldApplyBrightnessWsEvent()).toBe(false)

    session.dispose()
    registerBrightnessSession(null)
    expect(shouldApplyBrightnessWsEvent()).toBe(true)
  })
})
