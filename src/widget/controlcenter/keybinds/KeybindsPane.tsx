import { Gtk } from "ags/gtk4"
import { createState, createMemo, For } from "ags"

// Mock data until backend is ready
const mockBinds = [
    { name: "Terminal", description: "Open terminal", bind: "Super + Return", action: "exec kitty" },
    { name: "Close Window", description: "Close active window", bind: "Super + Q", action: "killactive" },
    { name: "Launcher", description: "Open application launcher", bind: "Super + Space", action: "exec rofi" },
]

export default function KeybindsPane(props: { name: string }) { // name prop for Stack
    const [searchText, setSearchText] = createState("")
    const [binds, setBinds] = createState(mockBinds)

    const filteredBinds = createMemo(() => {
        const query = searchText().toLowerCase()
        return binds().filter(b =>
            b.name.toLowerCase().includes(query) ||
            b.bind.toLowerCase().includes(query)
        )
    })

    return <box name={props.name} vertical={true} css="padding: 24px;" spacing={16}>
        <box spacing={12}>
            <label label="Keybinds" css="font-size: 24px; font-weight: bold;" />
            <box hexpand={true} />
            <entry
                placeholder_text="Search..."
                onChanged={({ text }) => setSearchText(text || "")}
                css="padding: 8px; border-radius: 8px; background-color: #313244;"
            />
        </box>

        <scrollable vexpand={true}>
            <box vertical={true} spacing={8}>
                <For each={filteredBinds}>
                    {(bind) => <box
                        css="background-color: #313244; padding: 12px; border-radius: 8px;"
                        spacing={12}
                    >
                        <box vertical={true}>
                            <label label={bind.name} xalign={0} css="font-weight: bold;" />
                            <label label={bind.description} xalign={0} css="font-size: 12px; color: #a6adc8;" />
                        </box>
                        <box hexpand={true} />
                        <label
                            label={bind.bind}
                            css="background-color: #45475a; padding: 4px 8px; border-radius: 4px; font-family: monospace;"
                        />
                    </box>}
                </For>
            </box>
        </scrollable>
    </box>
}
