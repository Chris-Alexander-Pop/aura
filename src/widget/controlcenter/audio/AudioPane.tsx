import { Gtk } from "ags/gtk4"
import { createState, createEffect } from "ags"
import sidecar from "../../../lib/sidecar"
import { AudioDevice, AudioStream } from "../../../lib/types"

export default function AudioPane(props: any) {
    const [sinks, setSinks] = createState<AudioDevice[]>([])
    const [sources, setSources] = createState<AudioDevice[]>([])
    const [streams, setStreams] = createState<AudioStream[]>([])

    const refresh = () => {
        sidecar.getAudioState().then(state => {
            setSinks(state.sinks)
            setSources(state.sources)
        }).catch(console.error)

        sidecar.getAudioStreams().then(setStreams).catch(console.error)
    }

    createEffect(() => {
        refresh()
        const interval = setInterval(refresh, 2000)
        return () => clearInterval(interval)
    })

    const DeviceRow = ({ device, type }: { device: AudioDevice, type: 'output' | 'input' }) => (
        <box spacing={12} css="margin-bottom: 8px;">
            <button
                onClicked={() => sidecar.setDefaultDevice(device.id, type).then(refresh)}
                css={`
                    min-width: 32px; min-height: 32px; 
                    border-radius: 16px;
                    background-color: ${device.is_default ? "#cba6f7" : "#313244"};
                    color: ${device.is_default ? "#11111b" : "#cdd6f4"};
                `}
            >
                <label
                    class="material-icon"
                    label={type === 'output' ? "volume_up" : "mic"}
                    css="font-family: 'Material Symbols Rounded';"
                />
            </button>
            <box orientation={Gtk.Orientation.VERTICAL} hexpand={true}>
                <label label={device.name} halign={Gtk.Align.START} css="font-weight: bold; font-size: 13px;" maxWidthChars={25} />
                <label label={device.info} halign={Gtk.Align.START} css="font-size: 11px; color: #a6adc8;" maxWidthChars={30} />
            </box>
        </box>
    )

    const StreamRow = ({ stream }: { stream: AudioStream }) => (
        <box spacing={12} css="margin-bottom: 8px;">
            <label
                class="material-icon"
                label="music_note"
                css="font-family: 'Material Symbols Rounded'; font-size: 20px; color: #a6adc8;"
            />
            <box orientation={Gtk.Orientation.VERTICAL} hexpand={true}>
                <label label={stream.name || stream.app} halign={Gtk.Align.START} css="font-weight: bold; font-size: 13px;" maxWidthChars={20} useMarkup={false} />
                <Gtk.Scale
                    hexpand={true}
                    orientation={Gtk.Orientation.HORIZONTAL}
                    adjustment={new Gtk.Adjustment({
                        lower: 0,
                        upper: 100,
                        value: stream.volume * 100
                    })}
                    digits={0}
                    onValueChanged={(self) => {
                        sidecar.setStreamVolume(stream.id, self.get_value() / 100)
                    }}
                />
            </box>
            <button
                onClicked={() => sidecar.setStreamMute(stream.id, stream.volume > 0)}
                css="padding: 4px;"
            >
                <label
                    class="material-icon"
                    label={stream.volume === 0 ? "volume_off" : "volume_up"}
                    css="font-family: 'Material Symbols Rounded';"
                />
            </button>
        </box>
    )

    return <Gtk.ScrolledWindow vexpand={true} css="padding: 16px;" {...props}>
        <box orientation={Gtk.Orientation.VERTICAL} spacing={24}>

            {/* Output Devices */}
            <box orientation={Gtk.Orientation.VERTICAL} spacing={8}>
                <label label="Output Devices" css="font-size: 14px; font-weight: bold; color: #fab387;" halign={Gtk.Align.START} />
                {sinks().map(dev => <DeviceRow device={dev} type="output" />)}
            </box>

            {/* Input Devices */}
            <box orientation={Gtk.Orientation.VERTICAL} spacing={8}>
                <label label="Input Devices" css="font-size: 14px; font-weight: bold; color: #fab387;" halign={Gtk.Align.START} />
                {sources().map(dev => <DeviceRow device={dev} type="input" />)}
            </box>

            {/* Applications */}
            {streams().length > 0 && <box orientation={Gtk.Orientation.VERTICAL} spacing={8}>
                <label label="Applications" css="font-size: 14px; font-weight: bold; color: #fab387;" halign={Gtk.Align.START} />
                {streams().map(stream => <StreamRow stream={stream} />)}
            </box>}

        </box>
    </Gtk.ScrolledWindow>
}
