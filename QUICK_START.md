# Quick Start Guide

## How the Sidecar Starts

**AGS automatically starts the sidecar** when it loads. You don't need to manually start it.

The SidecarClient (`src/services/sidecar.ts`) automatically:
1. Spawns `ags-sidecar server` process on import
2. Tries release binary first, falls back to debug
3. Auto-reconnects if connection fails

## Hot Reload Status

| Component | Hot Reload | Notes |
|-----------|------------|-------|
| **TypeScript** | ✅ Yes | Auto-rebuilds with `npm run watch` |
| **CSS** | ✅ Yes | Auto-rebuilds with `npm run watch` |
| **Rust Sidecar** | ❌ No | Requires rebuild + AGS auto-reconnects |

## Development Workflow

### Option 1: Simple (One Terminal)
```bash
npm run watch
```
- Watches TypeScript/CSS
- Auto-rebuilds and reloads AGS
- **For Rust changes**: Manually run `npm run build:sidecar` when needed

### Option 2: Full DX (Two Terminals) - Recommended

**Terminal 1: Frontend**
```bash
npm run watch
```

**Terminal 2: Backend (Rust hot-reload)**
```bash
# Install cargo-watch first (one-time)
cargo install cargo-watch

# Then watch Rust
cd sidecar
cargo watch -x 'build'  # Debug (faster builds)
```

**How it works:**
- Rust changes → `cargo-watch` rebuilds → AGS detects new binary → auto-reconnects
- TypeScript changes → Bun rebuilds → AGS reloads
- CSS changes → Tailwind rebuilds → AGS reloads

### Option 3: Unified Script
```bash
npm run dev
```
Builds sidecar first, then starts watch mode. Still need `cargo-watch` in separate terminal for Rust hot-reload.

## Testing Sidecar Without AGS

```bash
cd sidecar

# Test a method
cargo run -- client System.GetStats
cargo run -- client Power.GetBatteryState

# With JSON params
cargo run -- client Network.ScanNetworks '{}'
```

## Common Commands

```bash
# Build everything
npm run build:all

# Build just sidecar
npm run build:sidecar        # Release
npm run build:sidecar:debug  # Debug (faster)

# Watch mode
npm run watch                # TypeScript + CSS + AGS
npm run dev                  # Full setup (builds sidecar first)

# Test sidecar
npm run test:sidecar         # Run Rust tests
npm run test:sidecar:client  # Interactive CLI client
```

## Troubleshooting

**Sidecar won't start?**
- Build it: `npm run build:sidecar`
- Check it exists: `ls sidecar/target/release/ags-sidecar`

**Changes not showing?**
- TypeScript: Make sure `npm run watch` is running
- Rust: Rebuild sidecar (`npm run build:sidecar`) - AGS will auto-reconnect
- CSS: Should auto-rebuild with watch mode

**Want faster Rust builds?**
- Use debug mode: `npm run build:sidecar:debug`
- AGS will automatically use debug binary if release doesn't exist

## Pro Tips

1. **Use debug binary during development** - Faster builds, AGS auto-detects it
2. **Use cargo-watch for Rust** - Set it and forget it
3. **Test sidecar independently** - Use `cargo run -- client` to test methods
4. **Check AGS console** - Errors show connection issues clearly
