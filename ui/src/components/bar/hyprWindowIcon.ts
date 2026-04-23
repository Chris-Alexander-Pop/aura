/** Material icon name from Hyprland client class — strip + flyouts */
export function iconFromHyprClass(cls: string): string {
  const c = cls.toLowerCase()
  if (/firefox|chrome|chromium|brave|zen/.test(c)) return "public"
  if (/kitty|alacritty|foot|terminal|wezterm/.test(c)) return "terminal"
  if (/code|cursor|vscode|idea|jetbrains/.test(c)) return "code"
  if (/nautilus|thunar|files|dolphin/.test(c)) return "folder"
  if (/discord|slack|telegram/.test(c)) return "chat"
  if (/mpv|vlc/.test(c)) return "movie"
  return "window"
}
