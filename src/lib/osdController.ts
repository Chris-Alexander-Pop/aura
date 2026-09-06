export type OsdKind = "volume" | "brightness" | "mic"

export type OsdShowFn = (kind: OsdKind, label: string, percent: number) => void

const handlers = new Set<OsdShowFn>()

export function registerOsdHandler(fn: OsdShowFn): () => void {
    handlers.add(fn)
    return () => handlers.delete(fn)
}

export function showOsd(kind: OsdKind, label: string, percent: number) {
    for (const fn of handlers) {
        try {
            fn(kind, label, percent)
        } catch (e) {
            console.error("OSD handler error:", e)
        }
    }
}
