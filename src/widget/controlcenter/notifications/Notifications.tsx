import { Gtk } from "ags/gtk4"
import { bind } from "../../../lib/utils"
import Notifd, { Notification as Notif } from "gi://AstalNotifd"
import Notification from "./Notification"

export default function Notifications(props: any) {
    const notifd = Notifd.get_default()

    return <box
        class="Notifications"
        orientation={Gtk.Orientation.VERTICAL}
        spacing={8}
        {...props}
    >
        <box spacing={8} css="padding-bottom: 8px; border-bottom: 1px solid #313244;">
            <label
                label="Notifications"
                hexpand={true}
                halign={Gtk.Align.START}
                css="font-weight: bold; font-size: 16px;"
            />
            <button
                onClicked={() => {
                    // Notifd doesn't seem to have a clear_all method in the types we defined,
                    // but usually it mimics a dismissal loop or specific method.
                    // For now, let's iterate and dismiss.
                    notifd.notifications.forEach(n => n.dismiss())
                }}
                visible={bind(notifd, "notifications").as(n => n.length > 0)}
            >
                <Gtk.Image iconName="user-trash-symbolic" />
            </button>
        </box>

        <Gtk.ScrolledWindow vexpand={true} css="min-height: 200px;">
            <box orientation={Gtk.Orientation.VERTICAL} spacing={8}>
                {bind(notifd, "notifications").as(notifications => {
                    if (notifications.length === 0) {
                        return <box
                            vexpand={true}
                            halign={Gtk.Align.CENTER}
                            valign={Gtk.Align.CENTER}
                            orientation={Gtk.Orientation.VERTICAL}
                            spacing={8}
                            css="opacity: 0.5;"
                        >
                            <Gtk.Image iconName="notifications-disabled-symbolic" iconSize={Gtk.IconSize.LARGE} pixelSize={64} />
                            <label label="No Notifications" />
                        </box>
                    }

                    return notifications
                        .sort((a, b) => b.time - a.time)
                        .map(n => <Notification notification={n} />)
                })}
            </box>
        </Gtk.ScrolledWindow>
    </box>
}
