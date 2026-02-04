# Developer Experience Guide

## How the Sidecar Starts

### Current Behavior
**AGS automatically starts the sidecar** when the SidecarClient is imported. The connection happens in `src/services/sidecar.ts`:

1. On import, `sidecarClient.connect()` is called automatically
2. It spawns the sidecar process: `ags-sidecar server`
3. If the release binary doesn't exist, it falls back to debug binary
4. If connection fails, it auto-reconnects (up to 5 attempts)

### Manual Start (Optional)
You can also start the sidecar manually for testing:
```bash
cd /home/chris/.config/ags/sidecar
cargo run -- server
```

Or use the CLI client mode:
```bash
cargo run -- client System.GetStats
```

## Hot Reload Status

### TypeScript/AGS Frontend
✅ **Hot reload works** - AGS watch mode (`ags -w`) automatically reloads when `config.js` changes

### Rust Sidecar
❌ **No native hot reload** - Rust binaries need to be recompiled. However, we can set up tooling to make this smoother.

## Developer Experience Options

### Option 1: Manual Rebuild (Current)
```bash
# Terminal 1: Watch TypeScript/CSS
cd /home/chris/.config/ags
npm run watch

# Terminal 2: Rebuild sidecar when needed
cd /home/chris/.config/ags/sidecar
cargo build --release  # or cargo build for debug
# Then restart AGS (Ctrl+C and restart)
```

**Pros:** Simple, no extra dependencies
**Cons:** Manual, requires restarting AGS

### Option 2: Cargo Watch + Auto-restart (Recommended)
Use `cargo-watch` to automatically rebuild the sidecar, and AGS will reconnect automatically.

```bash
# Install cargo-watch
cargo install cargo-watch

# Watch and rebuild sidecar
cd /home/chris/.config/ags/sidecar
cargo watch -x 'build --release'
```

**Setup:**
- Terminal 1: `npm run watch` (AGS + TypeScript)
- Terminal 2: `cargo watch -x 'build --release'` (Rust sidecar)
- When Rust code changes, it rebuilds
- AGS auto-reconnects when it detects the new binary

**Pros:** Automatic rebuilds, AGS auto-reconnects
**Cons:** Requires cargo-watch installation

### Option 3: Unified Watch Script (Best DX)
Create a single script that watches both TypeScript and Rust.

**Pros:** One command, everything watches
**Cons:** More complex setup

### Option 4: Development Mode with Debug Binary
Use debug binary for faster builds during development:

```bash
# Build debug (faster)
cd /home/chris/.config/ags/sidecar
cargo build

# AGS will automatically use debug binary if release doesn't exist
```

**Pros:** Faster builds, easier debugging
**Cons:** Slower runtime performance

## Recommended Setup

### For Development
```bash
# Terminal 1: Watch everything
cd /home/chris/.config/ags
npm run dev  # (we'll create this)

# Terminal 2: Watch Rust (optional, if using cargo-watch)
cd /home/chris/.config/ags/sidecar
cargo watch -x 'build'
```

### For Production Testing
```bash
# Build release
cd /home/chris/.config/ags/sidecar
cargo build --release

# Run AGS
cd /home/chris/.config/ags
npm run watch
```

## Auto-Reconnection Behavior

The SidecarClient automatically:
1. **Detects disconnections** when the process exits
2. **Reconnects automatically** (up to 5 attempts with 2s delay)
3. **Falls back** from release to debug binary if needed
4. **Handles errors gracefully** - UI continues to work (just shows errors)

## Testing the Sidecar Independently

You can test the sidecar without AGS:

```bash
cd /home/chris/.config/ags/sidecar

# Test a method
cargo run -- client System.GetStats
cargo run -- client Power.GetBatteryState

# Run tests
cargo test

# Run with custom JSON
echo '{"jsonrpc":"2.0","method":"System.GetStats","id":1}' | cargo run -- server
```

## Improving DX

### Quick Wins
1. Add `cargo-watch` to development workflow
2. Create unified `npm run dev` script
3. Add sidecar health check endpoint
4. Better error messages in AGS when sidecar is down

### Future Enhancements
1. **Hot Module Replacement** - Use a development server that can reload Rust modules (complex)
2. **IPC over Unix Socket** - Faster than stdio, allows multiple clients
3. **Sidecar Health Dashboard** - Show connection status in AGS UI
4. **Automatic Binary Detection** - Auto-detect and use latest build
