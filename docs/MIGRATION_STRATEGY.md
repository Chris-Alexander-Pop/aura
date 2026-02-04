# Caelestia to AGS Migration Strategy

## Executive Summary
This document outlines the strategic approach for migrating the Caelestia shell from a Quickshell/QML codebase to an AGS/TypeScript + Rust Sidecar architecture. The primary goal is to maintain 100% feature parity ("literally everything") whilst improving performance, type safety, and maintainability.

## Architectural Shift

| Feature | Legacy (Caelestia/Quickshell) | New (AGS/Rust) | Benefit |
| :--- | :--- | :--- | :--- |
| **Frontend Language** | QML (JavaScript) | TypeScript (GJS) | Strong static typing, easier refactoring. |
| **UI Framework** | QtQuick | GTK3 (via AGS) | Native GTK integration, CSS styling (Tailwind). |
| **Backend Logic** | QML `Process` / Shell Scripts | Rust Sidecar (Binary) | High performance, memory safety, proper concurrency. |
| **IPC** | Stdout parsing / ad-hoc signals | JSON-RPC (Stdio/Unix Socket) | Structured, type-safe communication. |
| **Styling** | QML Properties | TailwindCSS | Industry standard styling, easier theming. |

## Migration Phases

### Phase 0: Discovery & Documentation (Current)
- Audit existing codebase.
- Map every QML file to a planned TypeScript component or Rust service.
- Define the JSON-RPC API spec.

### Phase 1: The "Brain" (Rust Sidecar)
Before writing any UI, we must build the backend services that power the shell. Reliability is key here.
- **Project Setup**: Cargo workspace.
- **Core Services**:
    - **VPN**: Port the complex state machine from `VPN.qml` (OpenConnect/OpenVPN management).
    - **Audio**: Pipewire integration (replacing `pw-cli` shell calls).
    - **Power**: Battery monitoring and TLP/Power-Profile-Daemon control.
    - **Network**: NetworkManager wrapper.
- **Testing**: Unit / integration testing is important to ensure that the sidecar works as intended and to prevent regressions
- **Deliverable**: A standalone `ags-sidecar` binary that can be queried via CLI (e.g., `ags-sidecar client get-volume`).

### Phase 2: Foundation (AGS Setup)
- Initialize AGS with `npm` and `typescript`.
- Configure `tailwind.config.js` with the Caelestia color palette.
- Implement the `SidecarClient` class in TypeScript to bridge the UI with the Rust binary.

### Phase 3: "Pixel Perfect" UI Porting
Porting components one by one, ensuring they look effectively identical.
1.  **Status Bar**: The anchor of the shell. Workspaces, clock, tray.
2.  **Notification Center**: Replicating the distinct notifications look.
3.  **Control Center**: interactable sliders and toggles backed by the Sidecar.
4.  **App Launcher**: Fast, keyboard-centric searching.

### Phase 4: Validation & Cutover
- **Parallel Run**: Run `ags -b debug` alongside Caelestia to compare visually.
- **Stress Test**: Hammer the RPC interface to ensure no lag.
- **Cutover**: Disable Caelestia autostart, enable AGS.

## Risk Management
- **Risk**: "GTK theming is harder than QML".
    - **Mitigation**: Use TailwindCSS purely; avoid complex GTK CSS where possible.
- **Risk**: "Rust sidecar crashes taking down shell features".
    - **Mitigation**: AGS should auto-restart the sidecar, or handle disconnects gracefully (grey out widgets).
- **Risk**: "VPN sudo permissions".
    - **Mitigation**: Use `polkit` properly or configure `sudoers` for specific commands (as Caelestia did), but orchestrated safely by Rust.
