// Caelestia-style design tokens — Catppuccin Mocha + Material 3

export const colors = {
    // Catppuccin Mocha base
    base: '#1e1e2e',
    mantle: '#181825',
    crust: '#11111b',
    surface0: '#313244',
    surface1: '#45475a',
    surface2: '#585b70',
    overlay0: '#6c7086',
    overlay1: '#7f849c',
    overlay2: '#9399b2',
    subtext0: '#a6adc8',
    subtext1: '#bac2de',
    text: '#cdd6f4',
    lavender: '#b4befe',
    blue: '#89b4fa',
    sapphire: '#74c7ec',
    sky: '#89dceb',
    teal: '#94e2d5',
    green: '#a6e3a1',
    yellow: '#f9e2af',
    peach: '#fab387',
    maroon: '#eba0ac',
    red: '#f38ba8',
    mauve: '#cba6f7',
    pink: '#f5c2e7',
    flamingo: '#f2cdcd',
    rosewater: '#f5e0dc',

    // Material 3 semantic roles (mapped to Catppuccin)
    m3primary: '#cba6f7',         // mauve
    m3onPrimary: '#11111b',       // crust
    m3primaryContainer: '#45475a', // surface1
    m3onPrimaryContainer: '#cba6f7',
    m3secondary: '#bac2de',        // subtext1
    m3onSecondary: '#11111b',
    m3secondaryContainer: '#313244', // surface0
    m3onSecondaryContainer: '#bac2de',
    m3tertiary: '#94e2d5',          // teal
    m3onTertiary: '#11111b',
    m3error: '#f38ba8',             // red
    m3onError: '#11111b',
    m3surface: '#1e1e2e',          // base
    m3surfaceContainer: '#181825',  // mantle
    m3surfaceContainerHigh: '#313244', // surface0 — OSD / elevated chrome
    m3surfaceContainerLow: '#11111b', // crust
    m3onSurface: '#cdd6f4',        // text
    m3onSurfaceVariant: '#a6adc8', // subtext0
    m3outline: '#6c7086',          // overlay0
    m3outlineVariant: '#45475a',   // surface1
} as const

export const spacing = {
    small: 7,
    smaller: 10,
    normal: 12,
    larger: 15,
    large: 20,
} as const

export const padding = {
    small: 5,
    smaller: 7,
    normal: 10,
    larger: 12,
    large: 15,
} as const

export const rounding = {
    small: 12,
    normal: 17,
    large: 25,
    full: 1000,
} as const

export const fonts = {
    sans: 'Rubik',
    mono: 'CaskaydiaCove NF',
    material: 'Material Symbols Rounded',
} as const

export const fontSize = {
    small: 11,
    smaller: 12,
    normal: 13,
    larger: 15,
    large: 18,
    extraLarge: 28,
} as const

export const bar = {
    innerWidth: 40,
    totalWidth: 56, // innerWidth + padding.normal * 2 - adjusted for GTK
    popoutWidth: {
        audio: 300,
        network: 340,
        battery: 250,
        bluetooth: 300,
    },
} as const

// Helper to generate inline CSS string for common patterns
export function materialIcon(size: number = fontSize.large) {
    return `font-family: '${fonts.material}'; font-size: ${size}px;`
}

export function surfaceContainer(radius: number = rounding.normal) {
    return `background-color: ${colors.m3surfaceContainer}; border-radius: ${radius}px;`
}
