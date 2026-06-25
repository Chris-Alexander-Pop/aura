/** Notify AGS that the pointer entered or left a hover-activated panel. */
export function postPanelHover(handler: string, hovered: boolean) {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(window as any).webkit?.messageHandlers?.[handler]?.postMessage?.(hovered ? "enter" : "leave")
  } catch {
    /* Vite dev server / non-WebKit */
  }
}
