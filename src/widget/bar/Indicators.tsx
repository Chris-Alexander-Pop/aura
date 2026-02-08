import { Gtk } from "ags/gtk4"
import App from "ags/gtk4/app"
import { createState, onMount, createMemo } from "ags"
import sidecar from "../../lib/sidecar"
import { AudioDevice, NetworkStatus } from "../../lib/types"

export default function Indicators() {
    const [volume, setVolume] = createState(0)
    const [muted, setMuted] = createState(false)
    const [wifi, setWifi] = createState(false)
    const [bt, setBt] = createState(false)

    onMount(() => {
        const updateAudio = () => {
            sidecar.getAudioState().then(s => {
                const def = s.sinks.find(d => d.is_default)
                if (def) {
                    setVolume(def.volume)
                    // We don't have a mute property in AudioDevice yet? 
                    // types.ts says: volume: number. 
                    // Usually 0 volume might mean muted, or we might need to update types/sidecar if mute is separate.
                    // For now, assume volume 0 is muted.
                    setMuted(def.volume === 0)
                }
            }).catch(console.error)
        }

        const updateNet = () => {
            sidecar.getNetworkStatus().then(s => {
                setWifi(s.active_connection ? true : false)
            }).catch(console.error)
        }

        const updateBt = () => {
            sidecar.getBluetoothAdapters().then(adapters => {
                setBt(adapters.some(a => a.powered))
            }).catch(console.error)
        }

        const update = () => {
            updateNet()
            updateAudio()
            updateBt()
        }

        update()
        // Poll every 2 seconds for snappier updates
        const interval = setInterval(update, 2000)
        return () => clearInterval(interval)
    })

    return <box class="Indicators" spacing={8} css="padding: 0 8px;">
        <button onClicked={() => App.toggle_window("control-center")}>
            <label
                class="material-icon"
                label={muted() ? "volume_off" : volume() > 0.5 ? "volume_up" : "volume_down"}
                css="font-family: 'Material Symbols Rounded';"
                tooltipText={`Volume: ${Math.round(volume() * 100)}%`}
            />
        </button>

        <button onClicked={() => App.toggle_window("control-center")} css="padding: 4px; border-radius: 8px; background-color: transparent; :hover { background-color: #313244; }">
            <label
                class="material-icon"
                label={wifi() ? "wifi" : "wifi_off"}
                css="font-family: 'Material Symbols Rounded';"
                tooltipText={wifi() ? "Connected" : "Disconnected"}
            />
        </button>

        <button onClicked={() => App.toggle_window("control-center")} css="padding: 4px; border-radius: 8px; background-color: transparent; :hover { background-color: #313244; }">
            <label
                class="material-icon"
                label={bt() ? "bluetooth" : "bluetooth_disabled"}
                css="font-family: 'Material Symbols Rounded';"
                tooltipText={bt() ? "Bluetooth On" : "Bluetooth Off"}
            />
        </button>
    </box>
}
