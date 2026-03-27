/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        // Catppuccin Mocha
        crust:    "#11111b",
        mantle:   "#181825",
        base:     "#1e1e2e",
        surface0: "#313244",
        surface1: "#45475a",
        surface2: "#585b70",
        overlay0: "#6c7086",
        overlay1: "#7f849c",
        overlay2: "#9399b2",
        subtext0: "#a6adc8",
        subtext1: "#bac2de",
        text:     "#cdd6f4",
        lavender: "#b4befe",
        blue:     "#89b4fa",
        sapphire: "#74c7ec",
        sky:      "#89dceb",
        teal:     "#94e2d5",
        green:    "#a6e3a1",
        yellow:   "#f9e2af",
        peach:    "#fab387",
        maroon:   "#eba0ac",
        red:      "#f38ba8",
        mauve:    "#cba6f7",
        pink:     "#f5c2e7",
        flamingo: "#f2cdcd",
        rosewater:"#f5e0dc",
      },
      fontFamily: {
        sans: ["Inter", "Rubik", "system-ui", "sans-serif"],
        mono: ["'CaskaydiaCove NF'", "'JetBrains Mono'", "monospace"],
        material: ["'Material Symbols Rounded'"],
      },
      borderRadius: {
        "4xl": "2rem",
      },
      backdropBlur: {
        "4xl": "64px",
      },
      animation: {
        "fade-in": "fadeIn 0.2s ease-out",
        "slide-in-right": "slideInRight 0.25s ease-out",
        "slide-in-left": "slideInLeft 0.25s ease-out",
        "slide-up": "slideUp 0.2s ease-out",
        "scale-in": "scaleIn 0.15s ease-out",
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        slideInRight: {
          "0%": { transform: "translateX(24px)", opacity: "0" },
          "100%": { transform: "translateX(0)", opacity: "1" },
        },
        slideInLeft: {
          "0%": { transform: "translateX(-24px)", opacity: "0" },
          "100%": { transform: "translateX(0)", opacity: "1" },
        },
        slideUp: {
          "0%": { transform: "translateY(16px)", opacity: "0" },
          "100%": { transform: "translateY(0)", opacity: "1" },
        },
        scaleIn: {
          "0%": { transform: "scale(0.92)", opacity: "0" },
          "100%": { transform: "scale(1)", opacity: "1" },
        },
      },
    },
  },
  plugins: [],
}
