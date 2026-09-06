/** @type {import('tailwindcss').Config} */
const withVar = (name) => `rgb(var(--c-${name}) / <alpha-value>)`

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        crust: withVar("crust"),
        mantle: withVar("mantle"),
        base: withVar("base"),
        surface0: withVar("surface0"),
        surface1: withVar("surface1"),
        surface2: withVar("surface2"),
        overlay0: withVar("overlay0"),
        overlay1: withVar("overlay1"),
        overlay2: withVar("overlay2"),
        subtext0: withVar("subtext0"),
        subtext1: withVar("subtext1"),
        text: withVar("text"),
        lavender: withVar("lavender"),
        blue: withVar("blue"),
        sapphire: withVar("sapphire"),
        sky: withVar("sky"),
        teal: withVar("teal"),
        green: withVar("green"),
        yellow: withVar("yellow"),
        peach: withVar("peach"),
        maroon: withVar("maroon"),
        red: withVar("red"),
        mauve: withVar("mauve"),
        pink: withVar("pink"),
        flamingo: withVar("flamingo"),
        rosewater: withVar("rosewater"),
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
