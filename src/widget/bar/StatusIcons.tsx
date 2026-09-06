import { Gtk } from "ags/gtk4"
import App from "ags/gtk4/app"
import { createState, onMount, onCleanup } from "ags"
import GLib from "gi://GLib"
import sidecar from "../../lib/sidecar"
import { colors, fonts } from "../../lib/theme"

function volumeIcon(vol: number, muted: boolean): string {
    if (muted || vol === 0) return "volume_off"
    if (vol > 0.66) return "volume_up"
    if (vol > 0.33) return "volume_down"
    return "volume_mute"
}

function networkIcon(connected: boolean): string {
    return connected ? "wifi" : "wifi_off"
}

function bluetoothIcon(powered: boolean, hasConnected: boolean): string {
    if (!powered) return "bluetooth_disabled"
    if (hasConnected) return "bluetooth_connected"
    return "bluetooth"
}

function batteryIcon(percent: number, charging: boolean): string {
    if (charging) {
        if (percent >= 90) return "battery_charging_full"
        if (percent >= 70) return "battery_charging_80"
        if (percent >= 50) return "battery_charging_60"
        if (percent >= 30) return "battery_charging_30"
        return "battery_charging_20"
    }
    if (percent >= 90) return "battery_full"
    if (percent >= 70) return "battery_6_bar"
    if (percent >= 50) return "battery_5_bar"
    if (percent >= 30) return "battery_3_bar"
    if (percent >= 10) return "battery_2_bar"
    return "battery_1_bar"
}

const MI = (text: string, color: string) =>
    `font-family: '${fonts.material}'; font-size: 18px; color: ${color};`

export default function StatusIcons() {
    const [volIcon, setVolIcon] = createState("volume_up")
    const [netIcon, setNetIcon] = createState("wifi")
    const [btIcon, setBtIcon] = createState("bluetooth")
    const [batIcon, setBatIcon] = createState("battery_full")
    const [batLow, setBatLow] = createState(false)
    const [volTip, setVolTip] = createState("Volume")
    const [netTip, setNetTip] = createState("Network")
    const [btTip, setBtTip] = createState("Bluetooth")
    const [batTip, setBatTip] = createState("Battery")

    onMount(() => {
        const update = () => {
            sidecar.getAudioState().then((s: any) => {
                const def = (s.sinks || []).find((d: any) => d.is_default)
                if (def) {
                    const muted = !!def.muted
                    setVolIcon(volumeIcon(def.volume, muted))
                    setVolTip(`Volume: ${muted ? "Muted" : `${Math.round(def.volume * 100)}%`}`)
                }
            }).catch(() => { })

            sidecar.getNetworkStatus().then((s: any) => {
                const c = !!s.active_connection
                setNetIcon(networkIcon(c))
                setNetTip(c ? "Connected" : "Not connected")
            }).catch(() => { })

            sidecar.getBluetoothAdapters().then((adapters: any[]) => {
                sidecar.getBluetoothDevices().then((devices: any[]) => {
                    const powered = (adapters || []).some((a: any) => a.powered)
                    const conn = (devices || []).some((d: any) => d.connected)
                    setBtIcon(bluetoothIcon(powered, conn))
                    setBtTip(powered ? (conn ? "BT: Connected" : "BT: On") : "BT: Off")
                }).catch(() => { })
            }).catch(() => { })

            sidecar.getBatteryState().then((s: any) => {
                if (s && s.percent !== undefined) {
                    setBatIcon(batteryIcon(s.percent, s.charging || false))
                    setBatLow(!s.charging && s.percent <= 20)
                    setBatTip(`Battery: ${s.percent}%`)
                }
            }).catch(() => { })
        }

        update()
        const audioHandler = () => update()
        sidecar.connect("audio-state", audioHandler)
        const id = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 2000, () => { update(); return true })
        onCleanup(() => {
            GLib.source_remove(id)
            sidecar.disconnect(audioHandler)
        })
    })

    const ic = colors.m3secondary

    return <box
        orientation={Gtk.Orientation.VERTICAL}
        halign={Gtk.Align.CENTER}
        css={`background-color: ${colors.m3surfaceContainer}; border-radius: 1000px; padding-top: 8px; padding-bottom: 8px;`}
    >
        {/* GTK popout windows were never registered in app.ts — use Control Center until dedicated popouts exist */}
        <button class="si-btn" onClicked={() => App.toggle_window("control-center")} tooltipText={volTip()}>
            <label label={volIcon()} css={MI(volIcon(), ic)} />
        </button>
        <button class="si-btn" onClicked={() => App.toggle_window("control-center")} tooltipText={netTip()}>
            <label label={netIcon()} css={MI(netIcon(), ic)} />
        </button>
        <button class="si-btn" onClicked={() => App.toggle_window("control-center")} tooltipText={btTip()}>
            <label label={btIcon()} css={MI(btIcon(), ic)} />
        </button>
        <button class="si-btn" onClicked={() => App.toggle_window("control-center")} tooltipText={batTip()}>
            <label label={batIcon()} css={MI(batIcon(), batLow() ? colors.m3error : ic)} />
        </button>
    </box>
}
