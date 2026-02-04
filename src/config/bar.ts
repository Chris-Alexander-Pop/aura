/**
 * Bar configuration – Phase 3 port of Caelestia BarConfig.
 * Controls status bar entries, workspaces, clock, tray, sizes.
 */
export const barConfig = {
    entries: [
        { id: 'workspaces', enabled: true },
        { id: 'spacer', enabled: true },
        { id: 'tray', enabled: true },
        { id: 'clock', enabled: true },
        { id: 'launcher', enabled: true },
        { id: 'power', enabled: true },
    ] as Array<{ id: string; enabled: boolean }>,
    workspaces: {
        shown: 5,
        perMonitorWorkspaces: true,
        activeIndicator: true,
        occupiedBg: false,
    },
    clock: {
        showIcon: true,
        useTwelveHour: false,
    },
    tray: {
        compact: false,
    },
    sizes: {
        innerWidth: 40,
    },
};
