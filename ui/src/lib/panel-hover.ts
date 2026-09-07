/** Notify AGS that the pointer entered or left a hover-activated panel. */
export function postPanelHover(handler: string, hovered: boolean) {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(window as any).webkit?.messageHandlers?.[handler]?.postMessage?.(hovered ? "enter" : "leave")
  } catch {
    /* Vite dev server / non-WebKit */
  }
}

/** Keep hover panels open while a slider thumb is dragged past the panel edge. */
export function scheduleLeaveAfterDrag(
  e: { buttons: number; currentTarget: HTMLElement },
  leave: () => void,
) {
  if (e.buttons !== 0) {
    const panel = e.currentTarget
    const onUp = (ev: PointerEvent) => {
      window.removeEventListener("pointerup", onUp, true)
      const r = panel.getBoundingClientRect()
      const inside =
        ev.clientX >= r.left &&
        ev.clientX <= r.right &&
        ev.clientY >= r.top &&
        ev.clientY <= r.bottom
      if (!inside) leave()
    }
    window.addEventListener("pointerup", onUp, true)
    return
  }
  leave()
}

