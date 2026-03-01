import { Gtk } from "ags/gtk4"
import { createState, onMount, onCleanup } from "ags"
import GLib from "gi://GLib"
import hyprland from "../../lib/hyprland"
import { colors, fonts } from "../../lib/theme"

function appIcon(cls: string): string {
    const c = cls.toLowerCase()
    if (c.includes("firefox") || c.includes("chrome") || c.includes("browser")) return "public"
    if (c.includes("terminal") || c.includes("kitty") || c.includes("alacritty") || c.includes("foot")) return "terminal"
    if (c.includes("code") || c.includes("cursor")) return "code"
    if (c.includes("file") || c.includes("nautilus") || c.includes("thunar")) return "folder"
    if (c.includes("discord") || c.includes("chat")) return "chat"
    if (c.includes("spotify") || c.includes("music")) return "music_note"
    if (c.includes("steam") || c.includes("game")) return "sports_esports"
    if (c.includes("settings")) return "settings"
    return "window"
}

export default function ActiveWindow() {
    const [icon, setIcon] = createState("window")
    const [title, setTitle] = createState("")
    const [hasWindow, setHasWindow] = createState(false)

    onMount(() => {
        const update = () => {
            try {
                const clients = hyprland.clients || []
                const focused = clients.find((c: any) => c.focus)
                if (focused) {
                    setIcon(appIcon(focused.class || ""))
                    setTitle(focused.title || focused.class || "")
                    setHasWindow(true)
                } else {
                    setHasWindow(false)
                }
            } catch {
                setHasWindow(false)
            }
        }

        // @ts-ignore
        hyprland.connect('focused-workspace-changed', update)
        update()

        const id = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1000, () => { update(); return true })
        onCleanup(() => GLib.source_remove(id))
    })

    return <box halign={Gtk.Align.CENTER} visible={hasWindow()} css="padding: 2px 0;">
        <label
            label={icon()}
            css={`font-family: '${fonts.material}'; font-size: 18px; color: ${colors.m3onSurfaceVariant};`}
            tooltipText={title()}
        />
    </box>
}
