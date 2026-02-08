import { Gtk } from "ags/gtk4"
import { createMemo } from "ags"
import { activeTab, setActiveTab, navExpanded, setNavExpanded } from "./ControlCenter"

const tabs = [
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
        class={createMemo(() => `nav-item ${isActive() ? "active" : ""}`)}
        onClicked={() => setActiveTab(id)}
        vexpand={false}
    >
        <box spacing={12}>
            <label
                class="material-icon"
                label={icon}
                css="font-family: 'Material Symbols Rounded'; font-size: 24px;"
            />
            {createMemo(() => navExpanded() ? <label label={id} /> : null)}
        </box>
    </button>
}

export default function NavRail() {
    return <box class="nav-rail" vertical={true} css="padding: 16px; background-color: #181825;">
        <button
            onClicked={() => setNavExpanded(!navExpanded())}
            css="margin-bottom: 16px;"
        >
            <label
                class="material-icon"
                label="menu"
                css="font-family: 'Material Symbols Rounded'; font-size: 24px;"
            />
        </button>

        <scrollable vexpand={true}>
            <box vertical={true} spacing={8}>
                {tabs.map(tab => <NavItem {...tab} />)}
            </box>
        </scrollable>
    </box>
}
