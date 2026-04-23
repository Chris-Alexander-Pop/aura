// DropdownTrigger — a thin invisible strip anchored to TOP CENTER.
// When the cursor enters it, the dropdown window is shown.
// When the cursor leaves the dropdown, it hides itself (handled via focus loss).

import { Astal, Gtk } from "ags/gtk4"
import App from "ags/gtk4/app"

export default function DropdownTrigger() {
    let hideTimer: ReturnType<typeof setTimeout> | null = null

    const clearHide = () => {
        if (hideTimer) {
            clearTimeout(hideTimer)
            hideTimer = null
        }
    }

    const scheduleHide = () => {
        clearHide()
        hideTimer = setTimeout(() => {
            App.get_window("dropdown")?.hide()
        }, 300)
    }

    return <window
        name="dropdown-trigger"
        anchor={Astal.WindowAnchor.TOP}
        layer={Astal.Layer.OVERLAY}
        exclusivity={Astal.Exclusivity.IGNORE}
        visible={true}
        application={App}
        css="background-color: transparent;"
    >
        <box
            css="min-width: 480px; min-height: 4px; background-color: transparent;"
            $={(self) => {
                const motion = Gtk.EventControllerMotion.new()
                motion.connect("enter", () => {
                    clearHide()
                    App.get_window("dropdown")?.show()
                })
                motion.connect("leave", () => {
                    scheduleHide()
                })
                self.add_controller(motion)
            }}
        />
    </window>
}
