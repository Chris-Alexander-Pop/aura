import { Gtk } from "ags/gtk4"
import { createMemo } from "ags"
import { activeTab } from "./ControlCenter"

// Placeholder for KeybindsPane - we will implement the real one next
import KeybindsPane from "./keybinds/KeybindsPane"

function PlaceholderPane({ title }: { title: string }) {
    return <box vertical={true} css="padding: 24px;">
        <label label={title} css="font-size: 24px; font-weight: bold;" halign={Gtk.Align.START} />
        <label label="Coming Soon..." css="color: #a6adc8;" halign={Gtk.Align.START} />
    </box>
}

export default function Panes() {
    return <stack
        visible_child_name={activeTab()}
        transition_type={Gtk.StackTransitionType.SLIDE_UP_DOWN}
        vexpand={true}
        hexpand={true}
    >
        <PlaceholderPane name="network" title="Network" />
        <PlaceholderPane name="bluetooth" title="Bluetooth" />
        <PlaceholderPane name="audio" title="Audio" />

        <KeybindsPane name="keybinds" />

        <PlaceholderPane name="performance" title="Performance" />
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
