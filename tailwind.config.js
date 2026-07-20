/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{ts,tsx,js,jsx}", "./app.ts"],
  theme: {
    extend: {},
  },
  // GTK4 CSS rejects these; Tailwind still emits the utilities unless blocked.
  // `contents` is also extracted from identifiers like `load_contents` in scanned TS.
  blocklist: [
    "visible",
    "invisible",
    "static",
    "fixed",
    "absolute",
    "relative",
    "sticky",
    "block",
    "inline",
    "inline-block",
    "flex",
    "inline-flex",
    "grid",
    "hidden",
    "contents",
    "table",
    "table-row",
    "table-cell",
    "flow-root",
    "list-item",
  ],
  plugins: [],
}
