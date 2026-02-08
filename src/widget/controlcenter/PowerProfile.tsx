import { Astal, Gtk, Gdk } from "ags/gtk4"
import { createState, createEffect, createMemo } from "ags"
import sidecar from "../../lib/sidecar"
// @ts-ignore
import Hyprland from "gi://AstalHyprland"

const hyprland = Hyprland.get_default()

export default function PowerProfile() {
    const [profile, setProfile] = createState<"performance" | "balanced" | "saver">("balanced")

    createEffect(() => {
        // Initial fetch
        sidecar.getPowerProfile().then(p => {
            if (p) setProfile(p as any)
        }).catch(console.error)
    })

    const applyProfile = (newProfile: "performance" | "balanced" | "saver") => {
        setProfile(newProfile)
        sidecar.setPowerProfile(newProfile).catch(console.error)

        // Visual Optimizations based on profile (replicating set-power-profile.sh)
        if (newProfile === "saver") {
            hyprland.message_async("keyword decoration:blur:enabled 0", null)
            hyprland.message_async("keyword decoration:drop_shadow:enabled 0", null)
            hyprland.message_async("keyword animations:enabled 0", null)
        } else {
            hyprland.message_async("keyword decoration:blur:enabled 1", null)
            hyprland.message_async("keyword decoration:drop_shadow:enabled 1", null)
            hyprland.message_async("keyword animations:enabled 1", null)
        }
    }

    const ProfileButton = ({ name, icon, value }: { name: string, icon: string, value: "performance" | "balanced" | "saver" }) => (
        <button
            class={createMemo(() => `p-2 rounded-lg transition-colors ${profile() === value ? "bg-[#cba6f7] text-[#11111b]" : "bg-[#313244] text-white hover:bg-[#45475a]"}`)}
            onClicked={() => applyProfile(value)}
            hexpand={true}
        >
            <box spacing={8} halign={Gtk.Align.CENTER}>
                <label class="font-material-symbols text-xl" label={icon} />
                <label label={name} css="font-weight: bold;" />
            </box>
        </button>
    )

    return <box orientation={Gtk.Orientation.VERTICAL} spacing={8} css="padding: 0 16px 16px 16px;">
        <label label="Power Profile" css="font-size: 14px; font-weight: bold; color: #fab387; margin-bottom: 8px;" halign={Gtk.Align.START} />
        <box spacing={8}>
            <ProfileButton name="Sa" icon="eco" value="saver" />
            <ProfileButton name="Ba" icon="balance" value="balanced" />
            <ProfileButton name="Pe" icon="speed" value="performance" />
        </box>
    </box>
}
