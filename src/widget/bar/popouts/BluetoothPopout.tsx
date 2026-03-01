import { Astal, Gtk } from "ags/gtk4"
import App from "ags/gtk4/app"
import { createState, onMount } from "ags"
import sidecar from "../../../lib/sidecar"
import { colors, fonts, bar } from "../../../lib/theme"

function deviceIcon(icon: string): string {
    if (!icon) return "bluetooth"
    if (icon.includes("audio") || icon.includes("headset") || icon.includes("headphone")) return "headphones"
    if (icon.includes("keyboard")) return "keyboard"
    if (icon.includes("mouse") || icon.includes("pointing")) return "mouse"
    if (icon.includes("phone")) return "smartphone"
    return "bluetooth"
}

const MI = (color: string, size: number = 18) => `font-family: '${fonts.material}'; font-size: ${size}px; color: ${color};`

export default function BluetoothPopout() {
    const anchor = Astal.WindowAnchor.LEFT | Astal.WindowAnchor.TOP | Astal.WindowAnchor.BOTTOM
    const [adapters, setAdapters] = createState<any[]>([])
    const [devices, setDevices] = createState<any[]>([])
    const [scanning, setScanning] = createState(false)

    const refresh = () => {
        sidecar.getBluetoothAdapters().then((a: any) => setAdapters(a || [])).catch(() => { })
        sidecar.getBluetoothDevices().then((d: any) => setDevices(d || [])).catch(() => { })
    }

    const scan = () => {
        setScanning(true)
        sidecar.scanBluetooth().then(() => { setTimeout(() => { refresh(); setScanning(false) }, 5000) }).catch(() => setScanning(false))
    }

    onMount(() => { refresh(); const i = setInterval(refresh, 5000); return () => clearInterval(i) })

    const powered = () => (adapters() || []).some((a: any) => a.powered)

    return <window
        name="popout-bluetooth"
        class="popout-window"
        anchor={anchor}
        application={App}
        visible={false}
        margin_left={56}
    >
        <box
            css={`background-color: ${colors.m3surface}; border-radius: 17px; padding: 15px; min-width: ${bar.popoutWidth.bluetooth}px;`}
            orientation={Gtk.Orientation.VERTICAL}
            spacing={8}
        >
            <box spacing={8}>
                <label label="Bluetooth" halign={Gtk.Align.START} hexpand={true}
                    css={`font-size: 14px; font-weight: 600; color: ${colors.text};`} />
                <button class={powered() ? "po-toggle on" : "po-toggle"}
                    onClicked={() => {
                        const a = adapters()[0]; if (!a) return
                        sidecar.setAdapterPower(a.path || a.adapter_path, !a.powered).then(refresh).catch(console.error)
                    }}>
                    <label label={powered() ? "On" : "Off"}
                        css={`font-size: 11px; font-weight: 500; color: ${powered() ? colors.crust : colors.text};`} />
                </button>
            </box>

            {devices().filter((d: any) => d.connected).length > 0 ? (
                <box orientation={Gtk.Orientation.VERTICAL} spacing={4}>
                    <label label="Connected" halign={Gtk.Align.START}
                        css={`font-size: 12px; font-weight: 600; color: ${colors.text};`} />
                    {devices().filter((d: any) => d.connected).map((dev: any) => (
                        <box css={`background-color: ${colors.surface0}; border-radius: 12px; padding: 10px;`} spacing={10}>
                            <label label={deviceIcon(dev.icon)} css={MI(colors.mauve, 20)} />
                            <box orientation={Gtk.Orientation.VERTICAL} hexpand={true}>
                                <label label={dev.name || dev.address} halign={Gtk.Align.START}
                                    css={`font-size: 13px; font-weight: 500; color: ${colors.text};`} />
                                <label label="Connected" halign={Gtk.Align.START}
                                    css={`font-size: 11px; color: ${colors.subtext0};`} />
                            </box>
                            <button class="icon-btn" onClicked={() => sidecar.disconnectDevice(dev.address).then(refresh).catch(console.error)}>
                                <label label="link_off" css={MI(colors.subtext0)} />
                            </button>
                        </box>
                    ))}
                </box>
            ) : <box />}

            <box css={`background-color: ${colors.surface1}; min-height: 1px; margin-top: 2px; margin-bottom: 2px;`} />

            <box spacing={4}>
                <label label="Available" halign={Gtk.Align.START} hexpand={true}
                    css={`font-size: 12px; font-weight: 600; color: ${colors.text};`} />
                {scanning() ? (
                    <label label="Scanning..." css={`font-size: 11px; color: ${colors.subtext0};`} />
                ) : (
                    <button class="icon-btn" onClicked={scan}>
                        <label label="refresh" css={MI(colors.subtext0, 16)} />
                    </button>
                )}
            </box>

            <Gtk.ScrolledWindow css="min-height: 150px;" vexpand={true}>
                <box orientation={Gtk.Orientation.VERTICAL} spacing={2}>
                    {devices().filter((d: any) => !d.connected).map((dev: any) => (
                        <button class="po-item" onClicked={() => sidecar.connectDevice(dev.address).then(refresh).catch(console.error)}>
                            <box spacing={10}>
                                <label label={deviceIcon(dev.icon)} css={MI(colors.subtext1)} />
                                <label label={dev.name || dev.address} hexpand={true} halign={Gtk.Align.START}
                                    css={`font-size: 12px; color: ${colors.text};`} ellipsize={3} maxWidthChars={25} />
                            </box>
                        </button>
                    ))}
                </box>
            </Gtk.ScrolledWindow>
        </box>
    </window>
}
