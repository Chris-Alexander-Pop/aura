# Caelestia Migration Feature Matrix

| Feature | Legacy Module | Sidecar Service | TS Client | UI Implementation | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **System** |
| Battery | [BatteryMonitor.qml](file:///home/chris/.config/quickshell/caelestia/modules/BatteryMonitor.qml) | [power.rs](file:///home/chris/.config/ags/sidecar/src/services/power.rs) | [getBatteryState](file:///home/chris/.config/ags/src/lib/sidecar.ts#167-169) | [Battery.tsx](file:///home/chris/.config/ags/src/widget/bar/Battery.tsx) | ✅ Done |
| Clock | `bar/Clock.qml` | [system.rs](file:///home/chris/.config/ags/sidecar/src/services/system.rs) | Local | [Clock.tsx](file:///home/chris/.config/ags/src/widget/bar/Clock.tsx) | ✅ Done |
| Workspaces | `bar/Workspaces.qml` | [hyprland.ts](file:///home/chris/.config/ags/src/lib/hyprland.ts) | IPC Wrapper | [Workspaces.tsx](file:///home/chris/.config/ags/src/widget/bar/Workspaces.tsx) | ✅ Done |
| **Connectivity** |
| Network | `controlcenter/network` | [network.rs](file:///home/chris/.config/ags/sidecar/src/services/network.rs) | [getNetworkStatus](file:///home/chris/.config/ags/src/lib/sidecar.ts#172-174) | ❌ | ⚠️ Backend Ready |
| Bluetooth | `controlcenter/bluetooth` | [bluetooth.rs](file:///home/chris/.config/ags/sidecar/src/services/bluetooth.rs) | [getBluetoothState](file:///home/chris/.config/ags/src/lib/sidecar.ts#192-194) | ❌ | ⚠️ Backend Ready |
| VPN | `controlcenter/vpn`? | [vpn.rs](file:///home/chris/.config/ags/sidecar/src/services/vpn.rs) | [getVpnStatus](file:///home/chris/.config/ags/src/lib/sidecar.ts#187-189) | ❌ | ⚠️ Backend Ready |
| **Media/Hardware** |
| Audio | `controlcenter/audio` | [audio.rs](file:///home/chris/.config/ags/sidecar/src/services/audio.rs) | [getAudioState](file:///home/chris/.config/ags/src/lib/sidecar.ts#183-185) | ❌ | ⚠️ Backend Ready |
| Brightness | `controlcenter/brightness`?| [brightness.rs](file:///home/chris/.config/ags/sidecar/src/services/brightness.rs) | [getBrightness](file:///home/chris/.config/ags/src/lib/sidecar.ts#179-181) | ❌ | ⚠️ Backend Ready |
| **Shell UI** |
| Launcher | `launcher/` | [packages.rs](file:///home/chris/.config/ags/sidecar/src/services/packages.rs) (Partial) | [getPackageUpdates](file:///home/chris/.config/ags/src/lib/sidecar.ts#219-221) | ❌ | ⚠️ Backend Ready |
| Control Center | `controlcenter/` | (Aggregated) | (Aggregated) | ❌ | ⚠️ Backend Ready |
| Notifications | `notifications/` | N/A | `notification` signal | ❌ | ⚠️ Part. Backend |
| OSD | `osd/` | N/A | N/A | ❌ | ⚠️ Pending |
| Session | `session/` | [power.rs](file:///home/chris/.config/ags/sidecar/src/services/power.rs) | [setPowerProfile](file:///home/chris/.config/ags/src/lib/sidecar.ts#170-171) | ❌ | ⚠️ Backend Ready |
| Weather | `weather/` | [weather.rs](file:///home/chris/.config/ags/sidecar/src/services/weather.rs) | [getWeather](file:///home/chris/.config/ags/src/lib/sidecar.ts#185-187) | ❌ | ⚠️ Backend Ready |
| Calendar | `calendar/` | [calendar.rs](file:///home/chris/.config/ags/sidecar/src/services/calendar.rs) | [getCalendarEvents](file:///home/chris/.config/ags/src/lib/sidecar.ts#213-215) | ❌ | ⚠️ Backend Ready |
| Logs | `logs/` | [logs.rs](file:///home/chris/.config/ags/sidecar/src/services/logs.rs) | [getLogs](file:///home/chris/.config/ags/src/lib/sidecar.ts#216-218) | ❌ | ⚠️ Backend Ready |
| Automation | `automation/` | [automation.rs](file:///home/chris/.config/ags/sidecar/src/services/automation.rs)| [triggerAutomation](file:///home/chris/.config/ags/src/lib/sidecar.ts#222-224) | ❌ | ⚠️ Backend Ready |
| Security | `security/` | [security.rs](file:///home/chris/.config/ags/sidecar/src/services/security.rs) | [getSecurityStatus](file:///home/chris/.config/ags/src/lib/sidecar.ts#204-206) | ❌ | ⚠️ Backend Ready |
| DevOps | `devops/` | [devops.rs](file:///home/chris/.config/ags/sidecar/src/services/devops.rs) | [getDevopsStatus](file:///home/chris/.config/ags/src/lib/sidecar.ts#207-209) | ❌ | ⚠️ Backend Ready |
| Fitness | `fitness/` | [fitness.rs](file:///home/chris/.config/ags/sidecar/src/services/fitness.rs) | [getFitnessGoals](file:///home/chris/.config/ags/src/lib/sidecar.ts#228-230) | ❌ | ⚠️ Backend Ready |
| Performance | `performance/` | [performance.rs](file:///home/chris/.config/ags/sidecar/src/services/performance.rs)| [getPerformanceMetrics](file:///home/chris/.config/ags/src/lib/sidecar.ts#201-203)| ❌ | ⚠️ Backend Ready |
| Keybinds | `keybinds/` | ❌ Missing | ❌ Missing | ❌ | 🔴 GAP |

## Gap Analysis
- **Critical Gap**: `Keybinds` tab exists in UI but has NO backend service.
- **Sidecar Client**: [src/lib/sidecar.ts](file:///home/chris/.config/ags/src/lib/sidecar.ts) exposes methods for ALL services effective EXCEPT Keybinds.
- **Signals**: Currently only `battery-state` and `power-profile` have specific signals. Other updates come through generic `notification` signal.
- **UI**: Only the Bar is implemented. The entire `Control Center`, `Launcher`, and `OSD` are missing.
