import GLib from "gi://GLib"
import Gtk from "gi://Gtk?version=4.0"
import { Gtk as GtkTags } from "ags/gtk4"
import { colors, fonts } from "../../lib/theme"

function exec(cmd: string) {
    GLib.spawn_command_line_async(cmd)
}

export default function Power() {
    const iconCss = `font-family: '${fonts.material}'; font-size: 18px; color: ${colors.m3error}; font-weight: bold;`

    return (
        <box halign={GtkTags.Align.CENTER}>
            <menubutton
                class="power-btn"
                $={(self: Gtk.MenuButton) => {
                    const pop = new Gtk.Popover()
                    const box = new Gtk.Box({
                        orientation: Gtk.Orientation.VERTICAL,
                        spacing: 4,
                    })
                    box.set_margin_top(10)
                    box.set_margin_bottom(10)
                    box.set_margin_start(10)
                    box.set_margin_end(10)

                    const row = (label: string, cmd: string) => {
                        const b = Gtk.Button.new_with_label(label)
                        b.add_css_class("po-item")
                        b.connect("clicked", () => {
                            exec(cmd)
                            pop.popdown()
                        })
                        box.append(b)
                    }

                    row("Lock", "loginctl lock-session")
                    row("Log out", "hyprctl dispatch exit")
                    row("Suspend", "systemctl suspend")
                    row("Restart", "systemctl reboot")
                    row("Power off", "systemctl poweroff")

                    pop.set_child(box)
                    self.set_popover(pop)
                }}
            >
                <label label="power_settings_new" css={iconCss} />
            </menubutton>
        </box>
    )
}
