import { Gtk } from "ags/gtk4"
import { createState, createEffect } from "ags"
import sidecar from "../../../lib/sidecar"
import { BluetoothDevice, BluetoothAdapter } from "../../../lib/types"

export default function BluetoothPane(props: any) {
    const [adapters, setAdapters] = createState<BluetoothAdapter[]>([])
    const [devices, setDevices] = createState<BluetoothDevice[]>([])
    const [scanning, setScanning] = createState(false)
    const [powered, setPowered] = createState(false)

    const refresh = () => {
        sidecar.getBluetoothAdapters().then(list => {
            setAdapters(list)
            if (list.length > 0) {
                setPowered(list[0].powered)
                setScanning(list[0].discovering)
            }
        }).catch(console.error)

        sidecar.getBluetoothDevices().then(setDevices).catch(console.error)
    }

    createEffect(() => {
        refresh()
        const interval = setInterval(refresh, 5000)
        return () => clearInterval(interval)
    })

    const togglePower = () => {
        if (adapters().length === 0) return
        const adapter = adapters()[0]
        sidecar.setAdapterPower(adapter.path, !adapter.powered).then(refresh).catch(console.error)
    }

    const toggleScan = () => {
        if (scanning()) {
            sidecar.stopScanBluetooth().then(refresh).catch(console.error)
        } else {
            sidecar.scanBluetooth().then(refresh).catch(console.error)
        }
    }

    return <box orientation={Gtk.Orientation.VERTICAL} css="padding: 16px;" spacing={16} {...props}>
        <box spacing={12} valign={Gtk.Align.CENTER}>
            <label
                label="Bluetooth"
                css="font-size: 18px; font-weight: bold;"
                hexpand={true}
                halign={Gtk.Align.START}
            />
            <button
                onClicked={toggleScan}
                css="padding: 4px 8px; border-radius: 6px;"
            >
                <label
                    class="material-icon"
                    label={scanning() ? "stop" : "refresh"}
                    css={`font-family: 'Material Symbols Rounded'; ${scanning() ? "color: #ff5555;" : ""}`}
                />
            </button>
            <switch
                active={powered()}
                onStateSet={togglePower}
            />
        </box>

        <Gtk.ScrolledWindow vexpand={true} css="min-height: 300px;">
            <box orientation={Gtk.Orientation.VERTICAL} spacing={8}>
                {devices().map(dev => (
                    <button
                        onClicked={() => {
                            if (dev.connected) {
                                sidecar.disconnectDevice(dev.address).then(refresh)
                            } else {
                                sidecar.connectDevice(dev.address).then(refresh)
                            }
                        }}
                        css={`
                            padding: 12px; 
                            background-color: ${dev.connected ? "#313244" : "transparent"};
                            border-radius: 8px;
                        `}
                    >
                        <box spacing={12}>
                            <label
                                class="material-icon"
                                label={dev.device_type === "audio-headset" ? "headphones" :
                                    dev.device_type === "input-mouse" ? "mouse" :
                                        dev.device_type === "input-keyboard" ? "keyboard" : "bluetooth"}
                                css="font-family: 'Material Symbols Rounded'; font-size: 20px;"
                            />
                            <box orientation={Gtk.Orientation.VERTICAL} valign={Gtk.Align.CENTER}>
                                <label label={dev.alias || dev.name} halign={Gtk.Align.START} css="font-weight: bold;" />
                                <label
                                    label={dev.address}
                                    halign={Gtk.Align.START}
                                    css="font-size: 12px; color: #a6adc8;"
                                />
                            </box>
                            {dev.connected && <label
                                class="material-icon"
                                label="link"
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
