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

  it("throttle_coalesces rapid setTarget into latest value", async () => {
    const sent: number[] = []
    const session = createBrightnessSession(async (frac) => {
      sent.push(frac)
      return { brightness: frac }
    })

    session.setTarget(0.2)
    expect(sent).toEqual([0.2])

    session.setTarget(0.4)
    session.setTarget(0.6)
    session.setTarget(0.8)
    expect(sent).toEqual([0.2])

    await vi.advanceTimersByTimeAsync(16)
    await Promise.resolve()

    expect(sent).toEqual([0.2, 0.8])
    expect(session.isBusy()).toBe(true)
  })

  it("throttle_sends_again after window elapses", async () => {
    const sent: number[] = []
    const session = createBrightnessSession(async (frac) => {
      sent.push(frac)
      return { brightness: frac }
    })

    session.setTarget(0.2)
    await vi.advanceTimersByTimeAsync(16)
    await Promise.resolve()

    session.setTarget(0.5)
    expect(sent).toEqual([0.2, 0.5])
  })

  it("flush_sends_immediately without waiting for throttle", async () => {
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
    const session = createBrightnessSession(async (frac) => ({ brightness: frac }))
    registerBrightnessSession(session)

    session.setTarget(0.4)
    expect(shouldApplyBrightnessWsEvent()).toBe(false)

    await vi.advanceTimersByTimeAsync(400)
    expect(shouldApplyBrightnessWsEvent()).toBe(true)

    session.dispose()
    registerBrightnessSession(null)
    expect(shouldApplyBrightnessWsEvent()).toBe(true)
  })
})
