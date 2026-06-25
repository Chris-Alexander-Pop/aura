import { Astal, Gtk, Gdk } from "ags/gtk4"
import App from "ags/gtk4/app"
import { createState, onMount, onCleanup } from "ags"
import GLib from "gi://GLib"
import sidecar from "../../lib/sidecar"
import { colors, fonts } from "../../lib/theme"
import { registerOsdHandler, showOsd, type OsdKind } from "../../lib/osdController"

const HIDE_MS = 2000

export default function OSD(gdkmonitor: Gdk.Monitor) {
    const anchor = Astal.WindowAnchor.BOTTOM | Astal.WindowAnchor.LEFT
    const [visible, setVisible] = createState(false)
    const [kind, setKind] = createState<OsdKind>("volume")
    const [label, setLabel] = createState("")
    const [pct, setPct] = createState(0)

    let hideId: number | null = null
    let coalesceId: number | null = null
    let pending: { kind: OsdKind; label: string; percent: number } | null = null
    let lastSinkKey = ""
    let lastSourceMuted: boolean | null = null

    const applyShow = (k: OsdKind, text: string, percent: number) => {
        setKind(k)
        setLabel(text)
        setPct(Math.max(0, Math.min(100, percent)))
        setVisible(true)
        if (hideId != null) GLib.source_remove(hideId)
        hideId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, HIDE_MS, () => {
            setVisible(false)
            hideId = null
            return false
        })
    }

    const show = (k: OsdKind, text: string, percent: number) => {
        pending = { kind: k, label: text, percent }
        if (coalesceId != null) return
        coalesceId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 40, () => {
            coalesceId = null
            if (pending) applyShow(pending.kind, pending.label, pending.percent)
            return false
        })
    }

    onMount(() => {
        const unregister = registerOsdHandler(show)

        const onAudio = (state: unknown) => {
            if (!state || typeof state !== "object") return
            const s = state as {
                sinks?: Array<{ is_default?: boolean; volume?: number; muted?: boolean }>
                sources?: Array<{ is_default?: boolean; muted?: boolean }>
            }
            const sink = (s.sinks || []).find((d) => d.is_default) ?? s.sinks?.[0]
            const source = (s.sources || []).find((d) => d.is_default) ?? s.sources?.[0]

            const sinkKey = sink
                ? `${sink.muted ? 1 : 0}:${Math.round((sink.volume ?? 0) * 100)}`
                : ""
            const sourceMuted = source?.muted ?? null
            const sinkChanged = sinkKey.length > 0 && sinkKey !== lastSinkKey
            const sourceChanged =
                sourceMuted !== null && sourceMuted !== lastSourceMuted

            if (sinkKey.length > 0) lastSinkKey = sinkKey
            if (sourceMuted !== null) lastSourceMuted = sourceMuted

            if (sourceChanged && !sinkChanged && source) {
                const muted = !!source.muted
                show("mic", muted ? "Mic muted" : "Mic on", muted ? 0 : 100)
                return
            }

            if (!sink) return
            const muted = !!sink.muted
            const p = Math.round((sink.volume ?? 0) * 100)
            show("volume", muted ? "Muted" : `${p}%`, muted ? 0 : p)
        }

        const onNotif = (method: string, params: unknown) => {
            if (method !== "Shell.Osd") return
            if (!params || typeof params !== "object") return
            const p = params as Record<string, unknown>
            if (p.kind === "brightness" || p.kind === "volume" || p.kind === "mic") {
                const k = p.kind as OsdKind
                const percent = typeof p.percent === "number" ? p.percent : 0
                const text = typeof p.label === "string" ? p.label : `${percent}%`
                show(k, text, percent)
            }
        }

        sidecar.connect("audio-state", onAudio)
        sidecar.connect("notification", onNotif)

        onCleanup(() => {
            unregister()
            sidecar.disconnect(onAudio)
            sidecar.disconnect(onNotif)
            if (hideId != null) GLib.source_remove(hideId)
            if (coalesceId != null) GLib.source_remove(coalesceId)
        })
    })

    const iconName = () => {
        if (kind() === "brightness") {
            const p = pct()
            if (p <= 25) return "brightness_2"
            if (p <= 50) return "brightness_4"
            if (p <= 75) return "brightness_6"
            return "brightness_7"
        }
        if (kind() === "mic") return pct() === 0 || label().toLowerCase().includes("mute") ? "mic_off" : "mic"
        return pct() === 0 || label().toLowerCase().includes("mute") ? "volume_off" : "volume_up"
    }

    return (
        <window
            name={`osd-${gdkmonitor.model}`}
            class="osd-window"
            gdkmonitor={gdkmonitor}
            visible={visible()}
            anchor={anchor}
            margin={20}
            application={App}
        >
            <box
                class="osd-container"
                orientation={Gtk.Orientation.HORIZONTAL}
                spacing={10}
                css={`background-color: ${colors.m3surfaceContainerHigh}; border-radius: 12px; padding: 12px 16px;`}
            >
                <label
                    label={iconName()}
                    css={`font-family: '${fonts.material}'; font-size: 22px; color: ${colors.m3onSurface};`}
                />
                <box orientation={Gtk.Orientation.VERTICAL} spacing={4}>
                    <label label={label()} css={`color: ${colors.m3onSurface}; font-size: 13px;`} />
                    <levelbar
                        value={pct() / 100}
                        widthRequest={160}
                        css="min-width: 160px;"
                        class="osd-scale"
                    />
                </box>
            </box>
        </window>
    )
}

/** Refresh OSD from sidecar state (called via `ags request osd …`). */
export async function refreshOsdFromSidecar(target: "volume" | "brightness" | "mic" | "all") {
    if (target === "volume" || target === "all") {
        try {
            const s = await sidecar.getAudioState()
            const def = (s.sinks || []).find((d: { is_default?: boolean }) => d.is_default)
            if (def) {
                const muted = !!(def as { muted?: boolean }).muted
                const p = Math.round(((def as { volume?: number }).volume ?? 0) * 100)
                showOsd("volume", muted ? "Muted" : `${p}%`, muted ? 0 : p)
            }
        } catch { /* sidecar offline */ }
    }

    if (target === "mic" || target === "all") {
        try {
            const s = await sidecar.getAudioState()
            const def = (s.sources || []).find((d: { is_default?: boolean }) => d.is_default)
            if (def) {
                const muted = !!(def as { muted?: boolean }).muted
                showOsd("mic", muted ? "Mic muted" : "Mic on", muted ? 0 : 100)
            }
        } catch { /* sidecar offline */ }
    }

    if (target === "brightness" || target === "all") {
        try {
            const b = await sidecar.getBrightness("active")
            const p = Math.round((b.brightness ?? 0.5) * 100)
            showOsd("brightness", `${p}%`, p)
        } catch { /* sidecar offline */ }
    }
}
