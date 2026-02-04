# Phase 2: AGS Foundation Setup - Complete

## Overview
Phase 2 establishes the foundation for the AGS frontend, including the SidecarClient for JSON-RPC communication and TailwindCSS configuration with Caelestia's Material Design 3 color palette.

## Completed Components

### 1. SidecarClient Implementation (`src/services/sidecar.ts`)
- **Full JSON-RPC 2.0 client** with request/response handling
- **Automatic reconnection** with exponential backoff
- **Notification support** for push updates from sidecar
- **Error handling** and timeout management
- **Convenience methods** for common operations (System, Power, Network, VPN, Audio)
- **Type-safe interfaces** for JSON-RPC messages

**Key Features:**
- Connects to sidecar binary (tries release, falls back to debug)
- Maintains persistent connection via stdin/stdout pipes
- Handles async request/response with proper ID tracking
- Supports notification subscriptions for real-time updates
- Auto-reconnects on connection failure (max 5 attempts)

### 2. TailwindCSS Configuration (`tailwind.config.js`)
- **Complete Material Design 3 color palette** from Caelestia
- **All M3 colors** including:
  - Surface colors (background, containers, variants)
  - Primary, Secondary, Tertiary color schemes
  - Error and Success colors
  - Inverse colors
  - Terminal colors (term-0 through term-15)
- **Transparency utilities** for layered effects
- **Ready for use** in all AGS components

### 3. Updated Main Application (`src/main.ts`)
- **Integrated SidecarClient** for system communication
- **Reactive variables** using AGS Variable system
- **Polling setup** for system stats and battery state
- **Notification handlers** for real-time updates
- **Example UI** using M3 colors from Tailwind

### 4. Build System
- **TypeScript compilation** via Bun
- **TailwindCSS build** script configured
- **Watch mode** for development (concurrently runs CSS watch + AGS watch)
- **CSS output** generated at `style/style.css`

## Usage

### Starting Development
```bash
cd /home/user/.config/ags
npm run watch
```

This will:
1. Watch for CSS changes and rebuild Tailwind
2. Watch for TypeScript changes and rebuild config.js
3. Run AGS in watch mode

### Building CSS Only
```bash
npm run build:css
```

### Building TypeScript
```bash
npm run build
```

## SidecarClient API

### Basic Request
```typescript
import { sidecar } from './services/sidecar';

// Generic request
const result = await sidecar.request('System.GetStats');

// Convenience methods
const stats = await sidecar.getStats();
const battery = await sidecar.getBatteryState();
```

### Notification Handling
```typescript
sidecar.onNotification('System.StatsUpdate', (notification) => {
    console.log('Stats updated:', notification.params);
});
```

### Available Convenience Methods
- `getStats()` - Get system statistics
- `getBatteryState()` - Get battery information
- `setPowerProfile(profile)` - Set power profile
- `toggleWifi(enabled)` - Toggle WiFi
- `scanNetworks()` - Scan for networks
- `connectVpn(profileId, auth)` - Connect VPN
- `disconnectVpn()` - Disconnect VPN
- `getVpnProfiles()` - Get VPN profiles
- `setVolume(stream, percent)` - Set audio volume
- `toggleMute(stream)` - Toggle mute

## Color Usage Examples

```typescript
// Background colors
className: 'bg-m3-surface'
className: 'bg-m3-surface-container'
className: 'bg-m3-primary-container'

// Text colors
className: 'text-m3-on-surface'
className: 'text-m3-primary'
className: 'text-m3-error'

// Borders
className: 'border-m3-outline'
className: 'border-m3-outline-variant'

// With transparency
className: 'bg-m3-surface-container/80'
className: 'border-m3-outline/35'
```

## Next Steps (Phase 3)
- Port Status Bar components
- Port Notification Center
- Port Control Center
- Port App Launcher

## Notes
- Sidecar binary path: `/home/user/.config/ags/sidecar/target/release/ags-sidecar`
- Debug fallback: `/home/user/.config/ags/sidecar/target/debug/ags-sidecar`
- CSS is generated from `style/input.css` to `style/style.css`
- TypeScript is compiled from `src/main.ts` to `config.js`
