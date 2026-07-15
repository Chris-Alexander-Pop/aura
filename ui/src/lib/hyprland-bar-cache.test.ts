import { describe, expect, it } from "vitest"
import { patchMonitorsActiveWorkspace } from "./hyprland-bar-cache"

describe("patchMonitorsActiveWorkspace", () => {
  it("updates the focused monitor active workspace", () => {
    const raw = [
      {
        name: "eDP-1",
        id: 0,
        focused: true,
        active_workspace: { id: 1, name: "1" },
      },
      {
        name: "HDMI-A-1",
        id: 1,
        focused: false,
        active_workspace: { id: 3, name: "3" },
      },
    ]
    const next = patchMonitorsActiveWorkspace(raw, 2, "2") as typeof raw
    expect(next[0]?.active_workspace).toEqual({ id: 2, name: "2" })
    expect(next[1]?.active_workspace).toEqual({ id: 3, name: "3" })
  })

  it("updates a named monitor even when unfocused", () => {
    const raw = [
      {
        name: "eDP-1",
        id: 0,
        focused: true,
        active_workspace: { id: 1, name: "1" },
      },
      {
        name: "HDMI-A-1",
        id: 1,
        focused: false,
        active_workspace: { id: 3, name: "3" },
      },
    ]
    const next = patchMonitorsActiveWorkspace(raw, 4, "4", "HDMI-A-1") as typeof raw
    expect(next[0]?.active_workspace).toEqual({ id: 1, name: "1" })
    expect(next[1]?.active_workspace).toEqual({ id: 4, name: "4" })
  })

  it("falls back to the sole monitor when focused is missing", () => {
    const raw = [
      {
        name: "eDP-1",
        id: 0,
        active_workspace: { id: 1, name: "1" },
      },
    ]
    const next = patchMonitorsActiveWorkspace(raw, 5, "5") as typeof raw
    expect(next[0]?.active_workspace).toEqual({ id: 5, name: "5" })
  })
})
