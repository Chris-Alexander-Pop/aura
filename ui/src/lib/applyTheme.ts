const THEME_ALIASES: Record<string, string> = {
  dark: "catppuccin-mocha",
  light: "catppuccin-latte",
}

export function normalizeThemeId(theme: string | undefined | null): string {
  const t = (theme ?? "dark").trim() || "dark"
  return THEME_ALIASES[t] ?? t
}

export function applyThemeToDocument(theme: string | undefined | null) {
  const id = normalizeThemeId(theme)
  document.documentElement.dataset.theme = id
}

export function isDarkTheme(theme: string | undefined | null): boolean {
  const id = normalizeThemeId(theme)
  return id !== "catppuccin-latte" && id !== "light"
}
