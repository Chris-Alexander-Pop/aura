import { Gtk } from "ags/gtk4"
import { createMemo } from "ags"
import { activeTab, setActiveTab, navExpanded, setNavExpanded } from "./ControlCenter"

const tabs = [
    { id: "notifications", icon: "notifications" },
    { id: "network", icon: "network_manage" },
    { id: "bluetooth", icon: "settings_bluetooth" },
    { id: "audio", icon: "tune" },
    { id: "keybinds", icon: "keyboard" },
    { id: "performance", icon: "speed" },
    { id: "security", icon: "security" },
    { id: "devops", icon: "deployed_code" },
    { id: "productivity", icon: "task" },
    { id: "calendar", icon: "calendar_month" },
    { id: "logs", icon: "receipt_long" },
    { id: "packages", icon: "extension" },
    { id: "automation", icon: "smart_toy" },
    { id: "communication", icon: "chat" },
    { id: "fitness", icon: "fitness_center" },
    { id: "weather", icon: "partly_cloudy_day" },
]

function NavItem({ id, icon }: { id: string, icon: string }) {
    const isActive = createMemo(() => activeTab() === id)

    return <button
        class={createMemo(() => `p-3 rounded-lg hover:bg-[#313244] ${isActive() ? "bg-[#cba6f7] text-[#11111b]" : "text-white"}`)}
        onClicked={() => setActiveTab(id)}
        vexpand={false}
    >
        <box spacing={12}>
            <label
                label={icon}
                class="font-material-symbols text-2xl"
            />
            {/* @ts-ignore */}
            {createMemo(() => navExpanded() ? <label label={id} /> : <box />)}
        </box>
    </button>
}

export default function NavRail() {
    return <box class="flex flex-col p-4 bg-[#181825]" orientation={Gtk.Orientation.VERTICAL}>
        <button
            onClicked={() => setNavExpanded(!navExpanded())}
            class="mb-4"
        >
            <label
                label="menu"
                class="font-material-symbols text-2xl"
            />
        </button>

        <Gtk.ScrolledWindow vexpand={true}>
            <box orientation={Gtk.Orientation.VERTICAL} spacing={8}>
                {tabs.map(tab => <NavItem {...tab} />)}
            </box>
        </Gtk.ScrolledWindow>
    </box>
}
