// WebSocket client for real-time sidecar push events (notifications, state changes)
import { create } from "zustand"
import { sidecarHttpToken } from "./sidecar-http"

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

function sidecarWsUrl(token: string): string {
  const proto =
    typeof window !== "undefined" && window.location.protocol === "https:" ? "wss" : "ws"
  const host =
    typeof window !== "undefined" && window.location.host
      ? window.location.host
      : "127.0.0.1:9080"
  return `${proto}://${host}/ws?token=${encodeURIComponent(token)}`
}

let socket: WebSocket | null = null
let reconnectTimer: ReturnType<typeof setTimeout> | null = null
let connecting = false

export function connectWs() {
  if (socket?.readyState === WebSocket.OPEN || connecting) return
  connecting = true

  void sidecarHttpToken()
    .then((token) => {
      connecting = false
      if (socket?.readyState === WebSocket.OPEN) return
      socket = new WebSocket(sidecarWsUrl(token))

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
    })
    .catch(() => {
      connecting = false
      reconnectTimer = setTimeout(connectWs, 3000)
    })
}

/** Subscribe to a specific sidecar push event by method name. */
export function useWsEvent(method: string, cb: (params: unknown) => void) {
  // Component-level hook usage — see pages for usage pattern
  return useWsStore.getState().on(method, cb)
}
