import { Astal, Gtk, Gdk } from "ags/gtk4"
import { createState, onMount, createEffect } from "ags"
import sidecar from "../../lib/sidecar"

export default function OSD(gdkmonitor: Gdk.Monitor) {
    const { BOTTOM } = Astal.WindowAnchor
    const [msg, setMsg] = createState<{ icon: string, value: number, label: string } | null>(null)
    const [visible, setVisible] = createState(false)

    let hideTimeout: any = null

    const show = (icon: string, val: number, label: string) => {
        setMsg({ icon, value: val, label })
        setVisible(true)
        if (hideTimeout) clearTimeout(hideTimeout)
        hideTimeout = setTimeout(() => setVisible(false), 2000)
    }

    onMount(() => {
        // Listen for triggers from sidecar signals
        // We assume sidecar emits 'volume-changed' or 'brightness-changed'
        // For now, let's mock it or connect if we added those signals.
        // If not, we might need to rely on polling or ensure sidecar sends notifications.

        // Connect to generic notification for now as a fallback if signals aren't granular
        // @ts-ignore
        sidecar.connect('notification', (method, params) => {
            if (method === 'Audio.VolumeChanged') {
                show('volume_up', params.volume * 100, `Volume: ${Math.round(params.volume * 100)}%`)
            }
            if (method === 'Brightness.Changed') {
                show('brightness_high', params.percent, `Brightness: ${params.percent}%`)
            }
        })
    })

    return <window
        name={`osd-${gdkmonitor.model}`}
        class="OSD"
        gdkmonitor={gdkmonitor}
        anchor={BOTTOM}
        visible={visible()}
        layer={Astal.Layer.OVERLAY}
        css="background-color: transparent;" // Let the box handle it
    >
        <box css="background-color: #1e1e2e; padding: 16px; border-radius: 12px; margin-bottom: 64px;">
            <box spacing={16} orientation={Gtk.Orientation.HORIZONTAL}>
                <label
                    class="material-icon"
                    label={msg()?.icon || "info"}
                    css="font-family: 'Material Symbols Rounded'; font-size: 24px;"
                />
                <box orientation={Gtk.Orientation.VERTICAL} valign={Gtk.Align.CENTER}>
                    <label label={msg()?.label || ""} css="font-weight: bold;" />
                    <levelbar
                        value={msg() ? msg()!.value / 100 : 0}
                        css="min-width: 150px;"
                    />
                </box>
            </box>
        </box>
    </window>
}
