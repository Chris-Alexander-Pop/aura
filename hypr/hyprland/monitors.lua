-- Per-monitor profiles (matched by EDID description).
-- Laptop logical width at 1.5 scale: 3840/1.5 = 2560 → external starts at 2560x0.

hl.monitor({
  output = "desc:AU Optronics 0x0000",
  mode = "3840x2160@60",
  position = "0x0",
  scale = 1.5,
})

-- HP M27fq QHD 27" — scale must be >= 1.0
hl.monitor({
  output = "desc:HP Inc. HP M27fq QHD EXAMPLESERIAL",
  mode = "2560x1440",
  position = "2560x0",
  scale = 1.0,
})

hl.monitor({
  output = "",
  mode = "preferred",
  position = "auto",
  scale = 1,
})
