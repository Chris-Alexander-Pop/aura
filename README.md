# AGS Configuration with Rust Sidecar

## Quick Start

### Development Mode (Aura launcher)
```bash
# From project root (builds sidecar, auto-watches Rust + frontend, starts AGS)
./aura

# Or via npm
npm run dev
```

**What `aura` does:**
- Checks dependencies (cargo, cargo-watch, npm/bun)
- Builds the sidecar (debug by default)
- Starts **cargo-watch** in the background (Rust auto-rebuild) when available
- Runs frontend watch (TypeScript/CSS) and AGS in the foreground
- Ctrl+C stops everything (including Rust watch)

**Options:**
- `aura --release` - Sidecar in release mode
- `aura --debug` - Sidecar in debug mode (default)
- `aura --no-rust-watch` - Don’t run cargo-watch (manual Rust rebuilds)
- `aura --help` - Show help

**Zsh alias (run from anywhere):**  
Add to `~/.zshrc`:
```bash
alias aura='~/.config/ags/aura'
```
Then run `aura` from any directory.

### Alternative: npm scripts
```bash
npm run dev        # Runs ./aura (full DX)
npm run dev:simple # Frontend watch only (sidecar must be built)
```

### Manual Setup
```bash
# 1. Build the sidecar
npm run build:sidecar        # Release (production)
# OR
npm run build:sidecar:debug  # Debug (faster builds, slower runtime)

# 2. Build TypeScript and CSS
npm run build
npm run build:css

# 3. Start AGS
ags -b my-ags-config
```

## Hot Reload

### TypeScript/CSS ✅
- **Automatic** - `npm run watch` watches for changes and rebuilds
- AGS automatically reloads when `config.js` changes

### Rust Sidecar
- **With `aura`**: Rust is auto-watched via `cargo-watch` in the background (install with `cargo install cargo-watch`).
- **Without cargo-watch**: Use `aura --no-rust-watch` and rebuild manually: `npm run build:sidecar:debug` or `build:sidecar`.
- AGS **auto-reconnects** when the sidecar binary is replaced after a rebuild.

## How It Works

### Sidecar Startup
1. **AGS automatically starts the sidecar** when `SidecarClient` is imported
2. Tries release binary first: `sidecar/target/release/ags-sidecar`
3. Falls back to debug: `sidecar/target/debug/ags-sidecar`
4. Auto-reconnects if connection fails (up to 5 attempts)

### Communication
- **Protocol**: JSON-RPC 2.0 over stdin/stdout
- **Format**: `{"jsonrpc":"2.0","method":"Service.Method","params":{},"id":1}`
- **Notifications**: Push updates from sidecar to AGS

## Development Workflow

### Best Practice

**Single terminal (recommended):**
```bash
aura
```
Runs Rust watch in the background and frontend watch + AGS in the foreground. One Ctrl+C stops all.

**Two terminals (optional):** Run `npm run watch` in one and `cd sidecar && cargo watch -x 'build'` in the other if you prefer separate logs.

### Testing Sidecar Independently
```bash
cd sidecar

# Test a method
cargo run -- client System.GetStats
cargo run -- client Power.GetBatteryState

# Run tests
cargo test
```

## Scripts

- `npm run build` - Build TypeScript to config.js
- `npm run build:css` - Build TailwindCSS
- `npm run build:sidecar` - Build sidecar (release)
- `npm run build:sidecar:debug` - Build sidecar (debug)
- `npm run build:all` - Build everything
- `npm run watch` - Watch TypeScript/CSS + start AGS
- `npm run dev` - Run `./aura` (builds sidecar, Rust watch + frontend watch)
- `npm run test:sidecar` - Run Rust tests

## Troubleshooting

### Sidecar won't start
- Check if binary exists: `ls sidecar/target/release/ags-sidecar`
- Build it: `npm run build:sidecar`
- Check logs in AGS console

### AGS can't connect
- Verify sidecar is running: `ps aux | grep ags-sidecar`
- Check binary permissions: `chmod +x sidecar/target/release/ags-sidecar`
- Try debug binary: `npm run build:sidecar:debug`

### Changes not reflecting
- **TypeScript**: Make sure `npm run watch` is running
- **Rust**: Rebuild sidecar and restart AGS (or use cargo-watch)
- **CSS**: Should auto-rebuild with watch mode

## Architecture

```
AGS (TypeScript)  ←→  JSON-RPC 2.0  ←→  Rust Sidecar
     (Frontend)         (stdin/stdout)      (Backend)
```

- **Frontend**: AGS widgets, UI logic, state management
- **Backend**: System calls, D-Bus, process management, data processing
- **Communication**: JSON-RPC 2.0 over stdio (can be upgraded to Unix socket)

## Next Steps

See `PHASE2_SETUP.md` for Phase 2 details and `DEVELOPER_EXPERIENCE.md` for full DX guide.
