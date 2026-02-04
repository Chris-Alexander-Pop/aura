/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{ts,js,tsx,jsx}", "./config.js"],
  theme: {
    extend: {
      colors: {
        // Material Design 3 Palette - Caelestia Theme
        // Base colors
        'm3-background': '#191114',
        'm3-on-background': '#efdfe2',
        'm3-surface': '#191114',
        'm3-surface-dim': '#191114',
        'm3-surface-bright': '#403739',
        'm3-surface-container-lowest': '#130c0e',
        'm3-surface-container-low': '#22191c',
        'm3-surface-container': '#261d20',
        'm3-surface-container-high': '#31282a',
        'm3-surface-container-highest': '#3c3235',
        'm3-on-surface': '#efdfe2',
        'm3-surface-variant': '#514347',
        'm3-on-surface-variant': '#d5c2c6',
        'm3-outline': '#9e8c91',
        'm3-outline-variant': '#514347',
        
        // Primary colors
        'm3-primary': '#ffb0ca',
        'm3-on-primary': '#541d34',
        'm3-primary-container': '#6f334a',
        'm3-on-primary-container': '#ffd9e3',
        'm3-primary-fixed': '#ffd9e3',
        'm3-primary-fixed-dim': '#ffb0ca',
        'm3-on-primary-fixed': '#39071f',
        'm3-on-primary-fixed-variant': '#6f334a',
        
        // Secondary colors
        'm3-secondary': '#e2bdc7',
        'm3-on-secondary': '#422932',
        'm3-secondary-container': '#5a3f48',
        'm3-on-secondary-container': '#ffd9e3',
        'm3-secondary-fixed': '#ffd9e3',
        'm3-secondary-fixed-dim': '#e2bdc7',
        'm3-on-secondary-fixed': '#2b151d',
        'm3-on-secondary-fixed-variant': '#5a3f48',
        
        // Tertiary colors
        'm3-tertiary': '#f0bc95',
        'm3-on-tertiary': '#48290c',
        'm3-tertiary-container': '#b58763',
        'm3-on-tertiary-container': '#000000',
        'm3-tertiary-fixed': '#ffdcc3',
        'm3-tertiary-fixed-dim': '#f0bc95',
        'm3-on-tertiary-fixed': '#2f1500',
        'm3-on-tertiary-fixed-variant': '#623f21',
        
        // Error colors
        'm3-error': '#ffb4ab',
        'm3-on-error': '#690005',
        'm3-error-container': '#93000a',
        'm3-on-error-container': '#ffdad6',
        
        // Success colors
        'm3-success': '#B5CCBA',
        'm3-on-success': '#213528',
        'm3-success-container': '#374B3E',
        'm3-on-success-container': '#D1E9D6',
        
        // Inverse colors
        'm3-inverse-surface': '#efdfe2',
        'm3-inverse-on-surface': '#372e30',
        'm3-inverse-primary': '#8b4a62',
        
        // Shadow and scrim
        'm3-shadow': '#000000',
        'm3-scrim': '#000000',
        'm3-surface-tint': '#ffb0ca',
        
        // Terminal colors (for compatibility)
        'term-0': '#353434',
        'term-1': '#ff4c8a',
        'term-2': '#ffbbb7',
        'term-3': '#ffdedf',
        'term-4': '#b3a2d5',
        'term-5': '#e98fb0',
        'term-6': '#ffba93',
        'term-7': '#eed1d2',
        'term-8': '#b39e9e',
        'term-9': '#ff80a3',
        'term-10': '#ffd3d0',
        'term-11': '#fff1f0',
        'term-12': '#dcbc93',
        'term-13': '#f9a8c2',
        'term-14': '#ffd1c0',
        'term-15': '#ffffff',
      },
      opacity: {
        'transparency-base': '0.88',
        'transparency-layer': '0.60',
      },
    },
  },
  plugins: [],
}
