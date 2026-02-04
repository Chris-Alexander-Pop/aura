/**
 * Control Center configuration – Phase 3 port of Caelestia control center.
 */
export const controlCenterConfig = {
    panes: [
        'network',
        'bluetooth',
        'audio',
        'performance',
        'power',
        'weather',
    ] as const,
    sizes: {
        heightMult: 0.7,
        ratio: 0.45,
    },
};

export type ControlCenterPane = (typeof controlCenterConfig.panes)[number];
