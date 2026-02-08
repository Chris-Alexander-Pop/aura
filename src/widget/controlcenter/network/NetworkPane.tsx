import { Gtk } from "ags/gtk4"
import { createState, createEffect } from "ags"
import sidecar from "../../../lib/sidecar"
import { AccessPoint, NetworkStatus } from "../../../lib/types"

export default function NetworkPane(props: any) {
    const [wifiEnabled, setWifiEnabled] = createState(true)
    const [networks, setNetworks] = createState<AccessPoint[]>([])
    const [status, setStatus] = createState<NetworkStatus | null>(null)
    const [scanning, setScanning] = createState(false)

    const refresh = () => {
        sidecar.getNetworkStatus().then(s => {
            setStatus(s)
            setWifiEnabled(s.wifi_enabled)
        }).catch(console.error)

        setScanning(true)
        sidecar.scanNetworks().then(nets => {
            setNetworks(nets)
            setScanning(false)
        }).catch(e => {
            console.error(e)
            setScanning(false)
        })
    }

    // Initial load and poll
    createEffect(() => {
        refresh()
        const interval = setInterval(refresh, 10000)
        return () => clearInterval(interval)
    })

    const toggleWifi = () => {
        const newState = !wifiEnabled()
        setWifiEnabled(newState)
        sidecar.toggleWifi(newState).then(refresh).catch(console.error)
    }

    return <box orientation={Gtk.Orientation.VERTICAL} css="padding: 16px;" spacing={16} {...props}>
        <box spacing={12} valign={Gtk.Align.CENTER}>
            <label
                label="Wi-Fi"
                css="font-size: 18px; font-weight: bold;"
                hexpand={true}
                halign={Gtk.Align.START}
            />
            <switch
                active={wifiEnabled()}
                onStateSet={({ active }) => toggleWifi()}
            />
        </box>

        <Gtk.ScrolledWindow vexpand={true} css="min-height: 300px;">
            <box orientation={Gtk.Orientation.VERTICAL} spacing={8}>
                {networks().map(ap => (
                    <button
                        onClicked={() => {
                            if (ap.active) return
                            // TODO: Input dialog for password
                            sidecar.connectNetwork(ap.ssid).then(refresh).catch(console.error)
                        }}
                        css={`
                            padding: 12px; 
                            background-color: ${ap.active ? "#313244" : "transparent"};
                            border-radius: 8px;
                        `}
                    >
                        <box spacing={12}>
                            <label
                                class="material-icon"
                                label={ap.strength > 75 ? "wifi" : ap.strength > 50 ? "wifi_2_bar" : "wifi_1_bar"}
                                css="font-family: 'Material Symbols Rounded'; font-size: 20px;"
                            />
                            <box orientation={Gtk.Orientation.VERTICAL} valign={Gtk.Align.CENTER}>
                                <label label={ap.ssid} halign={Gtk.Align.START} css="font-weight: bold;" />
                                <label
                                    label={`${ap.security} • ${ap.strength}%`}
                                    halign={Gtk.Align.START}
                                    css="font-size: 12px; color: #a6adc8;"
                                />
                            </box>
                            {ap.active && <label
                                class="material-icon"
                                label="check"
                                hexpand={true}
                                halign={Gtk.Align.END}
                                css="font-family: 'Material Symbols Rounded';"
                            />}
                        </box>
                    </button>
                ))}
            </box>
        </Gtk.ScrolledWindow>
    </box>
}
