import { Astal, Gtk } from "ags/gtk4"
import App from "ags/gtk4/app"
import { createState, onMount } from "ags"
import sidecar from "../../../lib/sidecar"
import { colors, fonts, bar } from "../../../lib/theme"

export default function AudioPopout() {
    const anchor = Astal.WindowAnchor.LEFT | Astal.WindowAnchor.TOP | Astal.WindowAnchor.BOTTOM
    const [sinks, setSinks] = createState<any[]>([])
    const [sources, setSources] = createState<any[]>([])
    const [volume, setVolume] = createState(0)
    const [defaultSinkId, setDefaultSinkId] = createState<number | null>(null)
    const [defaultSourceId, setDefaultSourceId] = createState<number | null>(null)

    const refresh = () => {
        sidecar.getAudioState().then((state: any) => {
            setSinks(state.sinks || [])
            setSources(state.sources || [])
            const defSink = (state.sinks || []).find((s: any) => s.is_default)
            const defSource = (state.sources || []).find((s: any) => s.is_default)
            if (defSink) { setDefaultSinkId(defSink.id); setVolume(defSink.volume) }
            if (defSource) setDefaultSourceId(defSource.id)
        }).catch(console.error)
    }

    onMount(() => { refresh(); const i = setInterval(refresh, 3000); return () => clearInterval(i) })

    const selectSink = (id: number) => {
        sidecar.setDefaultDevice(id, 'output').then(() => { setDefaultSinkId(id); refresh() }).catch(console.error)
    }
    const selectSource = (id: number) => {
        sidecar.setDefaultDevice(id, 'input').then(() => { setDefaultSourceId(id); refresh() }).catch(console.error)
    }

    const MI = (icon: string, color: string) => `font-family: '${fonts.material}'; font-size: 18px; color: ${color};`

    return <window
        name="popout-audio"
        class="popout-window"
        anchor={anchor}
        application={App}
        visible={false}
        margin_left={56}
    >
        <box
            css={`background-color: ${colors.m3surface}; border-radius: 17px; padding: 15px; min-width: ${bar.popoutWidth.audio}px;`}
            orientation={Gtk.Orientation.VERTICAL}
            spacing={8}
        >
            <label label="Audio" halign={Gtk.Align.START}
                css={`font-size: 14px; font-weight: 600; color: ${colors.text};`} />

            <box orientation={Gtk.Orientation.VERTICAL} spacing={4}>
                <label label={`Volume (${Math.round(volume() * 100)}%)`} halign={Gtk.Align.START}
                    css={`font-size: 12px; font-weight: 600; color: ${colors.text};`} />
                <slider class="po-slider" value={volume()}
                    onNotifyValue={(self: any) => {
                        const v = self.value; setVolume(v)
                        const d = defaultSinkId()
                        if (d !== null) sidecar.setStreamVolume(d, v).catch(console.error)
                    }} />
            </box>

            <box css={`background-color: ${colors.surface1}; min-height: 1px; margin-top: 4px; margin-bottom: 4px;`} />
            <label label="Output" halign={Gtk.Align.START}
                css={`font-size: 12px; font-weight: 600; color: ${colors.text};`} />
            <box orientation={Gtk.Orientation.VERTICAL} spacing={2}>
                {sinks().map((sink: any) => (
                    <button class="po-item" onClicked={() => selectSink(sink.id)}>
                        <box spacing={8}>
                            <label label={sink.id === defaultSinkId() ? "radio_button_checked" : "radio_button_unchecked"}
                                css={MI("", sink.id === defaultSinkId() ? colors.mauve : colors.overlay0)} />
                            <label label={sink.description || sink.name || "Unknown"}
                                css={`font-size: 12px; color: ${colors.text};`} ellipsize={3} maxWidthChars={28} />
                        </box>
                    </button>
                ))}
            </box>

            <box css={`background-color: ${colors.surface1}; min-height: 1px; margin-top: 4px; margin-bottom: 4px;`} />
            <label label="Input" halign={Gtk.Align.START}
                css={`font-size: 12px; font-weight: 600; color: ${colors.text};`} />
            <box orientation={Gtk.Orientation.VERTICAL} spacing={2}>
                {sources().map((source: any) => (
                    <button class="po-item" onClicked={() => selectSource(source.id)}>
                        <box spacing={8}>
                            <label label={source.id === defaultSourceId() ? "radio_button_checked" : "radio_button_unchecked"}
                                css={MI("", source.id === defaultSourceId() ? colors.mauve : colors.overlay0)} />
                            <label label={source.description || source.name || "Unknown"}
                                css={`font-size: 12px; color: ${colors.text};`} ellipsize={3} maxWidthChars={28} />
                        </box>
                    </button>
                ))}
            </box>
        </box>
    </window>
}
