import { Astal, Gtk, Gdk } from "ags/gtk4"
import GLib from "gi://GLib"
import App from "ags/gtk4/app"
import { createState, createEffect } from "ags"
import Apps, { Application } from "gi://AstalApps"

export default function Launcher() {
    const apps = new Apps()
    const [query, setQuery] = createState("")
    const [list, setList] = createState<Application[]>([])

    const hide = () => {
        App.toggle_window("launcher")
        setQuery("")
    }

    createEffect(() => {
        const text = query()
        if (text.startsWith("=")) {
            // Calculator mode
            try {
                // Safe eval or just eval for now since it's user input
                const result = String(eval(text.substring(1)))
                setList([{
                    app: null,
                    name: result,
                    description: "Calculator Result",
                    icon_name: "calc",
                    launch: () => {
                        App.toggle_window("launcher")
                        const display = Gdk.Display.get_default();
                        const clipboard = Gdk.Display.prototype.get_clipboard.call(display);
                        clipboard.set(result);
                    },
                    formatted: true
                } as any])
            } catch (e) {
                setList([{
                    app: null,
                    name: "Invalid Expression",
                    description: String(e),
                    icon_name: "dialog-error",
                    launch: () => { },
                    formatted: true
                } as any])
            }
        } else if (text.startsWith(">")) {
            // Runner mode
            const cmd = text.substring(1).trim()
            setList([{
                app: null,
                name: `Run: ${cmd}`,
                description: "Shell Command",
                icon_name: "utilities-terminal",
                launch: () => {
                    hide()
                    // execAsync is not available in ags v3 core apparently, using GLib directly
                    try {
                        GLib.spawn_command_line_async(cmd)
                    } catch (e) {
                        console.error(e)
                    }
                },
                formatted: true
            } as any])
        } else {
            // App search mode
            if (text === "") {
                setList(apps.get_list())
            } else {
                setList(apps.fuzzy_query(text))
            }
        }
    })

    const AppItem = ({ app }: { app: Application & { formatted?: boolean, launch: () => void } }) => (
        <button
            class="AppItem"
            onClicked={() => {
                if (!app.formatted) hide()
                app.launch()
            }}
            css="padding: 12px; border-radius: 8px; transition: all 0.2s;"
        >
            <box spacing={12}>
                <Gtk.Image iconName={app.icon_name || "application-x-executable"} iconSize={Gtk.IconSize.LARGE} pixelSize={32} />
                <box orientation={Gtk.Orientation.VERTICAL} valign={Gtk.Align.CENTER}>
                    <label
                        label={app.name}
                        halign={Gtk.Align.START}
                        css="font-weight: bold; font-size: 14px;"
                    />
                    <label
                        label={app.description || ""}
                        halign={Gtk.Align.START}
                        css="font-size: 11px; color: #a6adc8;"
                        ellipsize={3}
                        maxWidthChars={40}
                    />
                </box>
            </box>
        </button>
    )

    return <window
        name="launcher"
        application={App}
        visible={false}
        keymode={Astal.Keymode.EXCLUSIVE}
        anchor={Astal.WindowAnchor.TOP | Astal.WindowAnchor.BOTTOM | Astal.WindowAnchor.LEFT | Astal.WindowAnchor.RIGHT}
        // @ts-ignore
        onKeyPressed={(_, keyval) => {
            if (keyval === Gdk.KEY_Escape) hide()
        }}
    >
        <box css="padding: 80px;" halign={Gtk.Align.CENTER} valign={Gtk.Align.START}>
            <box
                orientation={Gtk.Orientation.VERTICAL}
                css="background-color: #1e1e2e; border-radius: 16px; padding: 24px; min-width: 500px;"
                spacing={16}
            >
                <entry
                    placeholderText="Search..."
                    text={query()}
                    onNotifyText={(self) => setQuery(self.text)}
                    onActivate={() => {
                        const first = list()[0]
                        if (first) {
                            hide()
                            first.launch()
                        }
                    }}
                    css="padding: 12px; font-size: 16px; border-radius: 12px; background-color: #313244; color: #cdd6f4;"
                />

                <Gtk.ScrolledWindow css="min-height: 400px;" vexpand={true}>
                    <box orientation={Gtk.Orientation.VERTICAL} spacing={8}>
                        {list().map(app => <AppItem app={app} />)}
                    </box>
                </Gtk.ScrolledWindow>
            </box>
        </box>
    </window>
}
