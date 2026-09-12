import { describe, expect, it } from "vitest"
import {
  parseHyprActiveWorkspace,
  parseHyprClients,
  parseHyprMonitors,
  parseHyprWorkspaces,
} from "./api-types"

const luaWorkspace = { address: "2", type: "numbered", name: "2" }

describe("hyprctl Lua-config workspace objects", () => {
  it("parses numbered workspaces from address/name when id is missing", () => {
    const list = parseHyprWorkspaces([
      { address: "1", type: "numbered", name: "1", windows: 2 },
      { address: "special:music", type: "special", name: "special:music", windows: 1 },
      { address: "3", type: "numbered", name: "3", windows: 0 },
    ])
    expect(list.map((w) => w.id)).toEqual([1, 3])
    expect(list[0]?.windows).toBe(2)
  })

  it("parses client workspace ids from address", () => {
    const clients = parseHyprClients([
      {
        address: "0xabc",
        class: "foot",
        title: "term",
        workspace: luaWorkspace,
      },
    ])
    expect(clients).toHaveLength(1)
    expect(clients[0]?.workspace).toEqual({ id: 2, name: "2" })
  })

  it("parses active workspace and monitor refs without id", () => {
    expect(parseHyprActiveWorkspace({ address: "4", type: "numbered", name: "4" })).toEqual({
      id: 4,
      name: "4",
    })
    const monitors = parseHyprMonitors([
      {
        id: 0,
        name: "eDP-1",
        focused: true,
        activeWorkspace: { address: "1", type: "numbered", name: "1" },
        specialWorkspace: { address: "special:special", type: "special", name: "special:special" },
      },
    ])
    expect(monitors[0]?.active_workspace).toEqual({ id: 1, name: "1" })
    expect(monitors[0]?.special_workspace).toEqual({ id: -1, name: "special:special" })
  })
})
