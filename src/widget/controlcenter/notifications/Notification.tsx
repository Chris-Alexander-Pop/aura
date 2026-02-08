import { Gtk } from "ags/gtk4"
import { bind } from "../../../lib/utils"
import Notifd, { Notification as Notif } from "gi://AstalNotifd"
import GLib from "gi://GLib"

function time(time: number, format = "%H:%M") {
    return GLib.DateTime.new_from_unix_local(time).format(format) || ""
}

export default function Notification({ notification }: { notification: Notif }) {
    return <box
        class={`Notification ${notification.urgency === 2 ? "critical" : ""}`}
        orientation={Gtk.Orientation.VERTICAL}
        spacing={8}
    >
        <box spacing={8}>
            <Gtk.Image
                iconName={notification.app_icon || notification.app_name || "dialog-information-symbolic"}
                iconSize={Gtk.IconSize.LARGE}
                pixelSize={32}
                valign={Gtk.Align.START}
            />
            <box orientation={Gtk.Orientation.VERTICAL} hexpand={true}>
                <box spacing={8}>
                    <label
                        class="app-name"
                        label={notification.app_name}
                        halign={Gtk.Align.START}
                        hexpand={true}
                        maxWidthChars={20}
                        ellipsize={3}
                        css="font-weight: bold; font-size: 11px; color: #a6adc8;"
                    />
                    <label
                        class="time"
                        label={time(notification.time)}
                        halign={Gtk.Align.END}
                        css="font-size: 11px; color: #a6adc8;"
                    />
                    <button
                        onClicked={() => notification.dismiss()}
                        valign={Gtk.Align.START}
                        css="padding: 0; background-color: transparent; border: none;"
                    >
                        <Gtk.Image iconName="window-close-symbolic" />
                    </button>
                </box>
                <label
                    class="summary"
                    label={notification.summary}
                    halign={Gtk.Align.START}
                    hexpand={true}
                    maxWidthChars={30}
                    wrap={true}
                    css="font-weight: bold; font-size: 14px;"
                />
                <label
                    class="body"
                    label={notification.body}
                    halign={Gtk.Align.START}
                    hexpand={true}
                    maxWidthChars={30}
                    wrap={true}
                    useMarkup={true} // Allow markup in body
                    css="color: #cdd6f4;"
                />
            </box>
        </box>

        {notification.actions.length > 0 && <box spacing={8} orientation={Gtk.Orientation.HORIZONTAL}>
            {notification.actions.map(action => (
                <button
                    onClicked={() => notification.invoke(action.id)}
                    hexpand={true}
                >
                    <label label={action.label} />
                </button>
            ))}
        </box>}
    </box>
}
