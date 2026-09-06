import type { PaneId } from "@/pages/control-center/navigation"
import { ALL_NAV_ITEMS } from "@/pages/control-center/navigation"

const STORAGE_KEY = "aura:control-center-pane"
const CHANNEL = "aura-control-center"

function isPaneId(value: string): value is PaneId {
  return ALL_NAV_ITEMS.some((item) => item.id === value)
}

/** Ask the control-center webview to show a specific pane (cross-window via BroadcastChannel). */
export function requestControlCenterPane(pane: PaneId) {
  try {
    localStorage.setItem(STORAGE_KEY, pane)
  } catch {
    /* private browsing */
  }
  try {
    new BroadcastChannel(CHANNEL).postMessage({ type: "open-pane", pane })
  } catch {
    /* unsupported */
  }
}

/** Read and clear a pane requested before the control-center webview was visible. */
export function consumeControlCenterPane(): PaneId | null {
  try {
    const pane = localStorage.getItem(STORAGE_KEY)
    localStorage.removeItem(STORAGE_KEY)
    if (pane && isPaneId(pane)) return pane
  } catch {
    /* private browsing */
  }
  return null
}

export function subscribeControlCenterPane(onPane: (pane: PaneId) => void): () => void {
  let bc: BroadcastChannel | null = null
  try {
    bc = new BroadcastChannel(CHANNEL)
    bc.onmessage = (event: MessageEvent<{ type?: string; pane?: string }>) => {
      const pane = event.data?.pane
      if (event.data?.type === "open-pane" && typeof pane === "string" && isPaneId(pane)) {
        onPane(pane)
      }
    }
  } catch {
    /* unsupported */
  }
  return () => bc?.close()
}
