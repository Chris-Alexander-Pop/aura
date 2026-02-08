import { Gtk } from "ags/gtk4"
import { createMemo } from "ags"
import { activeTab } from "./ControlCenter"

// Placeholder for KeybindsPane - we will implement the real one next
// Validated Panes
import KeybindsPane from "./keybinds/KeybindsPane"
import NetworkPane from "./network/NetworkPane"
import BluetoothPane from "./bluetooth/BluetoothPane"
import AudioPane from "./audio/AudioPane"
import Notifications from "./notifications/Notifications"
import SystemStats from "./SystemStats"
import PowerProfile from "./PowerProfile"

function PlaceholderPane({ title, ...props }: { title: string } & any) {
    return <box vertical={true} class="p-6" {...props}>
        <label label={title} class="text-2xl font-bold" halign={Gtk.Align.START} />
        <label label="Coming Soon..." class="text-[#a6adc8]" halign={Gtk.Align.START} />
    </box>
}

export default function Panes() {
    return <stack
        visible_child_name={activeTab()}
        transition_type={Gtk.StackTransitionType.SLIDE_UP_DOWN}
        vexpand={true}
        hexpand={true}
    >
        <NetworkPane name="network" />
        <BluetoothPane name="bluetooth" />
        <AudioPane name="audio" />
        <Notifications name="notifications" />

        <KeybindsPane name="keybinds" />

        <box name="performance" orientation={Gtk.Orientation.VERTICAL} css="padding: 16px;">
            <SystemStats />
            <PowerProfile />
        </box>
        <PlaceholderPane name="security" title="Security" />
        <PlaceholderPane name="devops" title="DevOps" />
        <PlaceholderPane name="productivity" title="Productivity" />
        <PlaceholderPane name="calendar" title="Calendar" />
        <PlaceholderPane name="logs" title="Logs" />
        <PlaceholderPane name="packages" title="Packages" />
        <PlaceholderPane name="automation" title="Automation" />
        <PlaceholderPane name="communication" title="Communication" />
        <PlaceholderPane name="fitness" title="Fitness" />
        <PlaceholderPane name="weather" title="Weather" />
    </stack>
}
