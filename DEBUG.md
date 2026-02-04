# Debugging Guide

## Quick Start

```bash
# Start development environment
./dev.sh

# In another terminal, watch Rust (if cargo-watch installed)
cd sidecar && cargo watch -x 'build'
```

## Debugging Options

### 1. Frontend (TypeScript/AGS) Debugging

**Check AGS logs:**
```bash
# AGS outputs to stderr, check your terminal or journal
journalctl -f -u ags
```

**Enable verbose logging:**
```bash
RUST_LOG=debug ags -b my-ags-config
```

**Test sidecar connection:**
- Check if sidecar process is running: `ps aux | grep ags-sidecar`
- Check connection in AGS console (F12 or check logs)

### 2. Backend (Rust Sidecar) Debugging

**Test sidecar independently:**
```bash
cd sidecar

# Test a method
cargo run -- client System.GetStats
cargo run -- client Power.GetBatteryState

# Run with debug logging
RUST_LOG=debug cargo run -- server
```

**Run tests:**
```bash
cd sidecar
cargo test
cargo test -- --nocapture  # Show output
```

**Check compilation:**
```bash
cd sidecar
cargo check        # Fast check
cargo build        # Debug build
cargo build --release  # Release build
```

### 3. Connection Issues

**Sidecar won't start:**
1. Check binary exists: `ls sidecar/target/release/ags-sidecar`
2. Build it: `./dev.sh` or `npm run build:sidecar`
3. Check permissions: `chmod +x sidecar/target/release/ags-sidecar`

**AGS can't connect:**
1. Check sidecar is running: `ps aux | grep ags-sidecar`
2. Check AGS logs for connection errors
3. Try manual start: `cd sidecar && cargo run -- server`
4. Test with CLI client: `cd sidecar && cargo run -- client System.GetStats`

**Auto-reconnect not working:**
- Check AGS console for errors
- Verify sidecar binary path in `src/services/sidecar.ts`
- Check file permissions

### 4. Build Issues

**TypeScript errors:**
```bash
npm run build  # See errors
```

**Rust compilation errors:**
```bash
cd sidecar
cargo check  # Fast check
cargo build  # Full build with errors
```

**CSS not updating:**
```bash
npm run build:css  # Manual rebuild
```

## Common Issues

### "Sidecar not connected"
- Build the sidecar: `npm run build:sidecar`
- Check binary path in `src/services/sidecar.ts`
- Verify sidecar starts: `cd sidecar && cargo run -- server`

### "Method not found"
- Check method name matches exactly (case-sensitive)
- Verify service is registered in `sidecar/src/services/mod.rs`
- Check sidecar logs for errors

### "Request timeout"
- Sidecar might be stuck
- Kill and restart: `pkill ags-sidecar`
- Check for infinite loops or blocking operations

### Changes not reflecting
- **TypeScript**: Make sure `npm run watch` is running
- **Rust**: Rebuild sidecar - AGS will auto-reconnect
- **CSS**: Should auto-rebuild, try `npm run build:css`

## Debug Scripts

The `dev.sh` script provides:
- `./dev.sh` - Debug build + watch
- `./dev.sh --release` - Release build + watch
- `./dev.sh --no-rust-watch` - Skip Rust watching

## Profiling

**Rust sidecar:**
```bash
cd sidecar
cargo build --release --profile release-with-debug
perf record --call-graph dwarf ./target/release/ags-sidecar server
```

**TypeScript:**
- Use browser devtools if running in debug mode
- Check AGS console output

## Logging

**Enable Rust logging:**
```bash
RUST_LOG=debug cargo run -- server
RUST_LOG=sidecar::services=debug cargo run -- server  # Specific module
```

**AGS logging:**
- Check terminal output
- Check `~/.local/share/ags/logs/` if AGS creates logs
- Use `journalctl` for system logs
