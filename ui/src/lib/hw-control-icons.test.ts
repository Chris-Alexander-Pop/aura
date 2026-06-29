import { describe, expect, it } from "vitest"
import { brightnessIcon, volumeIcon } from "./hw-control-icons"

describe("volumeIcon", () => {
  it("returns off/mute/down/up tiers", () => {
    expect(volumeIcon(0, false)).toBe("volume_off")
    expect(volumeIcon(50, true)).toBe("volume_off")
    expect(volumeIcon(10, false)).toBe("volume_mute")
    expect(volumeIcon(50, false)).toBe("volume_down")
    expect(volumeIcon(80, false)).toBe("volume_up")
  })
})

describe("brightnessIcon", () => {
  it("matches OSD brightness tiers", () => {
    expect(brightnessIcon(10)).toBe("brightness_2")
    expect(brightnessIcon(40)).toBe("brightness_4")
    expect(brightnessIcon(60)).toBe("brightness_6")
    expect(brightnessIcon(90)).toBe("brightness_7")
  })
})
