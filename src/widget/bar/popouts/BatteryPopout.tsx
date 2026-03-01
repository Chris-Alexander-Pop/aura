import { Astal, Gtk } from "ags/gtk4"
import App from "ags/gtk4/app"
import { createState, onMount } from "ags"
import sidecar from "../../../lib/sidecar"
import { colors, fonts, bar } from "../../../lib/theme"

function getBatteryIcon(p: number, c: boolean): string {
    if (c) {
        if (p >= 90) return "battery_charging_full"; if (p >= 70) return "battery_charging_80"
        if (p >= 50) return "battery_charging_60"; if (p >= 30) return "battery_charging_30"
        return "battery_charging_20"
    }
    if (p >= 90) return "battery_full"; if (p >= 70) return "battery_6_bar"
    if (p >= 50) return "battery_5_bar"; if (p >= 30) return "battery_3_bar"
    if (p >= 10) return "battery_2_bar"; return "battery_1_bar"
}

const profiles = [
    { id: "saver", label: "Saver", icon: "energy_savings_leaf" },
    { id: "balanced", label: "Balanced", icon: "balance" },
    { id: "performance", label: "Perform", icon: "rocket_launch" },
]

export default function BatteryPopout() {
    const anchor = Astal.WindowAnchor.LEFT | Astal.WindowAnchor.TOP | Astal.WindowAnchor.BOTTOM
    const [pct, setPct] = createState(100)
    const [chg, setChg] = createState(false)
    const [timeRem, setTimeRem] = createState("")
    const [prof, setProf] = createState("balanced")

    const refresh = () => {
        sidecar.getBatteryState().then((s: any) => {
            if (s) { setPct(s.percent ?? 100); setChg(s.charging ?? false); setTimeRem(s.time_remaining || "") }
        }).catch(() => { })
        sidecar.getPowerProfile().then((p: any) => {
            if (typeof p === 'string') setProf(p); else if (p?.profile) setProf(p.profile)
        }).catch(() => { })
    }

    onMount(() => { refresh(); const i = setInterval(refresh, 5000); return () => clearInterval(i) })

    return <window
        name="popout-battery"
        class="popout-window"
        anchor={anchor}
        application={App}
        visible={false}
        margin_left={56}
    >
        <box
            css={`background-color: ${colors.m3surface}; border-radius: 17px; padding: 15px; min-width: ${bar.popoutWidth.battery}px;`}
            orientation={Gtk.Orientation.VERTICAL}
            spacing={12}
        >
            <box spacing={12} halign={Gtk.Align.CENTER}>
                <label label={getBatteryIcon(pct(), chg())}
                    css={`font-family: '${fonts.material}'; font-size: 36px; color: ${pct() <= 20 && !chg() ? colors.m3error : colors.mauve};`} />
                <box orientation={Gtk.Orientation.VERTICAL} valign={Gtk.Align.CENTER}>
                    <label label={`${pct()}%`} css={`font-size: 24px; font-weight: 700; color: ${colors.text};`} />
                    <label label={chg() ? "Charging" : (timeRem() || "On battery")}
                        css={`font-size: 11px; color: ${colors.subtext0};`} />
                </box>
            </box>

            <box css={`background-color: ${colors.surface1}; min-height: 1px;`} />

            <label label="Power profile" halign={Gtk.Align.START}
                css={`font-size: 12px; font-weight: 600; color: ${colors.text};`} />

            <box spacing={6} halign={Gtk.Align.CENTER} homogeneous={true}>
                {profiles.map(p => (
                    <button class={prof() === p.id ? "profile-chip active" : "profile-chip"}
                        onClicked={() => sidecar.setPowerProfile(p.id as any).then(() => setProf(p.id)).catch(console.error)}>
                        <box orientation={Gtk.Orientation.VERTICAL} spacing={4} halign={Gtk.Align.CENTER}>
                            <label label={p.icon}
                                css={`font-family: '${fonts.material}'; font-size: 20px; color: ${prof() === p.id ? colors.crust : colors.subtext1};`} />
                            <label label={p.label}
                                css={`font-size: 10px; font-weight: 500; color: ${prof() === p.id ? colors.crust : colors.subtext0};`} />
                        </box>
                    </button>
                ))}
            </box>
        </box>
    </window>
}
