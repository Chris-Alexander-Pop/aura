import { describe, expect, it } from "vitest"
import {
  iconFromSpecialWorkspace,
  isSpecialWorkspaceOpen,
  specialCoverByWorkspaceId,
  specialWorkspaceCovers,
  specialWorkspaceLabel,
  specialWorkspaceToggleArg,
} from "./special-workspace"

describe("special-workspace", () => {
  it("detects open special workspaces from hyprctl monitor refs", () => {
    expect(isSpecialWorkspaceOpen({ id: 0, name: "" })).toBe(false)
    expect(isSpecialWorkspaceOpen({ id: -98, name: "special:communication" })).toBe(true)
    expect(isSpecialWorkspaceOpen({ id: -1, name: "special:special" })).toBe(true)
  })

  it("maps names to toggle args and icons", () => {
    expect(specialWorkspaceLabel("special:communication")).toBe("communication")
    expect(specialWorkspaceToggleArg("special:communication")).toBe("communication")
    expect(iconFromSpecialWorkspace("special:communication")).toBe("chat")
    expect(iconFromSpecialWorkspace("special:music")).toBe("music_note")
    expect(iconFromSpecialWorkspace("special:grok")).toBe("smart_toy")
  })

  it("collects covered workspace ids across all monitors", () => {
    const covers = specialWorkspaceCovers([
      {
        name: "eDP-1",
        id: 0,
        active_workspace: { id: 1, name: "1" },
        special_workspace: { id: -98, name: "special:communication" },
      },
      {
        name: "HDMI-A-1",
        id: 1,
        active_workspace: { id: 2, name: "2" },
        special_workspace: { id: 0, name: "" },
      },
    ])
    expect(covers).toHaveLength(1)
    expect(covers[0]?.workspaceId).toBe(1)
    const byWs = specialCoverByWorkspaceId(covers)
    expect(byWs.get(1)?.special.name).toBe("special:communication")
  })
})
