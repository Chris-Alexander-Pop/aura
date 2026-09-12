import { describe, expect, it } from "vitest"
import {
  collectOccupiedWorkspaceIds,
  pickVisibleWorkspaces,
} from "./workspace-visible-range"

describe("collectOccupiedWorkspaceIds", () => {
  it("uses client workspace ids", () => {
    expect(collectOccupiedWorkspaceIds([1, 4, 1, 0, -98])).toEqual([1, 4])
  })

  it("falls back to hypr workspace window counts when clients are empty", () => {
    expect(
      collectOccupiedWorkspaceIds(
        [],
        [
          { id: 1, windows: 2 },
          { id: 2, windows: 0 },
          { id: 3, windows: 1 },
        ],
      ),
    ).toEqual([1, 3])
  })

  it("unions clients with workspace window counts", () => {
    expect(
      collectOccupiedWorkspaceIds(
        [2],
        [
          { id: 1, windows: 3 },
          { id: 2, windows: 0 },
        ],
      ),
    ).toEqual([1, 2])
  })
})

describe("pickVisibleWorkspaces", () => {
  it("returns nothing when there is no occupancy and no active workspace", () => {
    expect(pickVisibleWorkspaces([], null)).toEqual([])
  })

  it("always includes the active workspace even when empty", () => {
    expect(pickVisibleWorkspaces([], 6)).toEqual([6])
    expect(pickVisibleWorkspaces([1, 2], 6)).toEqual([1, 2, 6])
  })

  it("shows every occupied workspace when under the cap", () => {
    expect(pickVisibleWorkspaces([1, 2, 5], 1)).toEqual([1, 2, 5])
  })
})
