import { Astal, Gtk } from "ags/gtk4"
import App from "ags/gtk4/app"
import { createState, onMount } from "ags"
import sidecar from "../../../lib/sidecar"
import { colors, fonts, bar } from "../../../lib/theme"

function signalIcon(strength: number): string {
    if (strength >= 75) return "signal_wifi_4_bar"
    if (strength >= 50) return "network_wifi_3_bar"
    if (strength >= 25) return "network_wifi_2_bar"
    return "network_wifi_1_bar"
}

const MI = (color: string, size: number = 18) => `font-family: '${fonts.material}'; font-size: ${size}px; color: ${color};`

export default function NetworkPopout() {
    const anchor = Astal.WindowAnchor.LEFT | Astal.WindowAnchor.TOP | Astal.WindowAnchor.BOTTOM
    const [connected, setConnected] = createState(false)
    const [activeAp, setActiveAp] = createState("")
    const [networks, setNetworks] = createState<any[]>([])
    const [scanning, setScanning] = createState(false)
    const [wifiOn, setWifiOn] = createState(true)

    const refresh = () => {
        sidecar.getNetworkStatus().then((s: any) => {
            setConnected(!!s.active_connection)
            setActiveAp(s.active_connection?.name || s.active_connection || "")
            setWifiOn(s.wifi_enabled !== false)
        }).catch(() => { })
    }

    const scan = () => {
        setScanning(true)
        sidecar.scanNetworks().then((aps: any[]) => { setNetworks(aps || []); setScanning(false) }).catch(() => setScanning(false))
    }

    onMount(() => { refresh(); scan(); const i = setInterval(() => { refresh(); scan() }, 10000); return () => clearInterval(i) })

    return <window
        name="popout-network"
        class="popout-window"
        anchor={anchor}
        application={App}
        visible={false}
        margin_left={56}
    >
        <box
            css={`background-color: ${colors.m3surface}; border-radius: 17px; padding: 15px; min-width: ${bar.popoutWidth.network}px;`}
            orientation={Gtk.Orientation.VERTICAL}
            spacing={8}
        >
            <box spacing={8}>
                <label label="Network" halign={Gtk.Align.START} hexpand={true}
                    css={`font-size: 14px; font-weight: 600; color: ${colors.text};`} />
                <button class={wifiOn() ? "po-toggle on" : "po-toggle"}
                    onClicked={() => { const n = !wifiOn(); sidecar.toggleWifi(n).then(() => { setWifiOn(n); if (n) scan() }).catch(console.error) }}>
                    <label label={wifiOn() ? "Wi-Fi On" : "Wi-Fi Off"}
                        css={`font-size: 11px; font-weight: 500; color: ${wifiOn() ? colors.crust : colors.text};`} />
                </button>
            </box>

            {connected() ? (
                <box css={`background-color: ${colors.surface0}; border-radius: 12px; padding: 10px;`} spacing={10}>
                    <label label="wifi" css={MI(colors.mauve, 20)} />
                    <box orientation={Gtk.Orientation.VERTICAL} hexpand={true}>
                        <label label={activeAp()} halign={Gtk.Align.START} css={`font-size: 13px; font-weight: 500; color: ${colors.text};`} />
                        <label label="Connected" halign={Gtk.Align.START} css={`font-size: 11px; color: ${colors.subtext0};`} />
                    </box>
                    <button class="icon-btn" onClicked={() => sidecar.disconnectNetwork().then(refresh).catch(console.error)}>
                        <label label="link_off" css={MI(colors.subtext0)} />
                    </button>
                </box>
            ) : (
                <box css={`background-color: ${colors.surface0}; border-radius: 12px; padding: 10px;`} spacing={10}>
                    <label label="wifi_off" css={MI(colors.subtext0, 20)} />
                    <label label="Not connected" css={`font-size: 13px; color: ${colors.subtext0};`} />
                </box>
            )}

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

            <Gtk.ScrolledWindow css="min-height: 200px;" vexpand={true}>
                <box orientation={Gtk.Orientation.VERTICAL} spacing={2}>
                    {networks().filter((n: any) => n.ssid && n.ssid !== activeAp()).map((net: any) => (
                        <button class="po-item" onClicked={() => sidecar.connectNetwork(net.ssid).then(refresh).catch(console.error)}>
                            <box spacing={10}>
                                <label label={signalIcon(net.signal || net.strength || 0)} css={MI(colors.subtext1)} />
                                <label label={net.ssid} hexpand={true} halign={Gtk.Align.START}
                                    css={`font-size: 12px; color: ${colors.text};`} ellipsize={3} maxWidthChars={25} />
                                {net.security && net.security !== "none" ? (
                                    <label label="lock" css={MI(colors.overlay0, 14)} />
                                ) : <box />}
                            </box>
                        </button>
                    ))}
                </box>
            </Gtk.ScrolledWindow>
        </box>
    </window>
}
