// WebSocket client for real-time sidecar push events (notifications, state changes)
import { create } from "zustand"

export interface SidecarEvent {
  method: string
  params: unknown
}

interface WsStore {
  connected: boolean
  lastEvent: SidecarEvent | null
  listeners: Map<string, Set<(params: unknown) => void>>
  setConnected: (v: boolean) => void
  setLastEvent: (e: SidecarEvent) => void
  on: (method: string, cb: (params: unknown) => void) => () => void
  emit: (e: SidecarEvent) => void
}

export const useWsStore = create<WsStore>((set, get) => ({
  connected: false,
  lastEvent: null,
  listeners: new Map(),

  setConnected: (v) => set({ connected: v }),
  setLastEvent: (e) => set({ lastEvent: e }),

  on: (method, cb) => {
    const { listeners } = get()
    if (!listeners.has(method)) listeners.set(method, new Set())
    listeners.get(method)!.add(cb)
    return () => listeners.get(method)?.delete(cb)
  },

  emit: (e) => {
    const { listeners } = get()
    listeners.get(e.method)?.forEach((cb) => cb(e.params))
    listeners.get("*")?.forEach((cb) => cb(e))
    set({ lastEvent: e })
  },
}))

const WS_URL = "ws://localhost:9080/ws"
let socket: WebSocket | null = null
let reconnectTimer: ReturnType<typeof setTimeout> | null = null

export function connectWs() {
  if (socket?.readyState === WebSocket.OPEN) return

  socket = new WebSocket(WS_URL)

  socket.onopen = () => {
    useWsStore.getState().setConnected(true)
    if (reconnectTimer) clearTimeout(reconnectTimer)
  }

  socket.onmessage = (ev) => {
    try {
      const msg = JSON.parse(ev.data) as SidecarEvent
      useWsStore.getState().emit(msg)
    } catch {
      console.warn("[ws] invalid message", ev.data)
    }
  }

  socket.onclose = () => {
    useWsStore.getState().setConnected(false)
    reconnectTimer = setTimeout(connectWs, 3000)
  }

  socket.onerror = () => {
    socket?.close()
  }
}

/** Subscribe to a specific sidecar push event by method name. */
export function useWsEvent(method: string, cb: (params: unknown) => void) {
  // Component-level hook usage — see pages for usage pattern
  return useWsStore.getState().on(method, cb)
}
