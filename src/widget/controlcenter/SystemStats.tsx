import { Gtk } from "ags/gtk4"
import { createState, createEffect } from "ags"
import sidecar from "../../lib/sidecar"

export default function SystemStats() {
    const [cpu, setCpu] = createState(0)
    const [ram, setRam] = createState(0)
    const [temp, setTemp] = createState(0)

    createEffect(() => {
        const update = () => {
            sidecar.getSystemStats().then(stats => {
                setCpu(stats.cpu)
                setRam(stats.ram)
                setTemp(stats.temp)
            }).catch(console.error)
        }

        update()
        const interval = setInterval(update, 2000)
        return () => clearInterval(interval)
    })

    const StatBar = ({ label, value, icon, color }: { label: string, value: number, icon: string, color: string }) => (
        <box spacing={12} css="background-color: #313244; padding: 12px; border-radius: 8px;">
            <label class="material-icon font-material-symbols text-xl" label={icon} css={`color: ${color};`} />
            <box orientation={Gtk.Orientation.VERTICAL} hexpand={true} valign={Gtk.Align.CENTER} spacing={4}>
                <box>
                    <label label={label} css="font-weight: bold; font-size: 12px;" halign={Gtk.Align.START} />
                    <box hexpand={true} />
                    <label label={`${Math.round(value)}%`} css="font-size: 12px; color: #a6adc8;" halign={Gtk.Align.END} />
                </box>
                <levelbar
                    value={value / 100}
                    hexpand={true}
                    css={`
                        min-height: 4px;
                        block.filled { background-color: ${color}; border-radius: 2px; }
                        block.empty { background-color: #45475a; border-radius: 2px; }
                    `}
                />
            </box>
        </box>
    )

    return <box orientation={Gtk.Orientation.VERTICAL} spacing={8} css="padding: 0 16px 16px 16px;">
        <label label="System" css="font-size: 14px; font-weight: bold; color: #fab387; margin-bottom: 8px;" halign={Gtk.Align.START} />
        <StatBar label="CPU" value={cpu()} icon="memory" color="#eba0ac" />
        <StatBar label="RAM" value={ram()} icon="memory_alt" color="#89b4fa" />
        <StatBar label="Temp" value={temp()} icon="thermostat" color="#f9e2af" />
    </box>
}
