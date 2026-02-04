/**
 * Status Bar – Phase 3 port of Caelestia bar.
 * Workspaces, clock, system tray, power.
 * Pass Hyprland and SystemTray from Service.import() when enabling the UI.
 */
import Window from 'resource:///com/github/Aylur/ags/widget/window.js';
import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import CenterBox from 'resource:///com/github/Aylur/ags/widget/centerbox.js';
import { barConfig } from '../../config/bar';
import { Workspaces, type HyprlandLike } from './Workspaces';
import { Clock } from './Clock';
import { Tray, type SystemTrayLike } from './Tray';
import { LauncherButton } from './LauncherButton';
import { Power } from './Power';

function Spacer() {
    return Box({
        className: 'min-w-4',
        hexpand: true,
    });
}

export function Bar(hyprland?: HyprlandLike | null, systemTray?: SystemTrayLike | null) {
    const entries = barConfig.entries.filter((e) => e.enabled);
    const left: unknown[] = [];
    const center: unknown[] = [];
    const right: unknown[] = [];

    for (const e of entries) {
        if (e.id === 'spacer') {
            if (left.length > 0 && right.length === 0) center.push(Spacer());
            else if (left.length > 0) right.push(Spacer());
            else left.push(Spacer());
            continue;
        }
        if (e.id === 'workspaces') left.push(Workspaces(hyprland));
        else if (e.id === 'clock') center.push(Clock());
        else if (e.id === 'tray') right.push(Tray(systemTray));
        else if (e.id === 'launcher') right.push(LauncherButton());
        else if (e.id === 'power') right.push(Power());
    }

    return Window({
        name: 'bar',
        anchor: ['top', 'left', 'right'],
        exclusivity: 'exclusive',
        child: Box({
            className: 'bar-root p-2 bg-m3-surface/80 border-b border-m3-outline/30',
            children: [
                CenterBox({
                    startWidget: Box({
                        className: 'gap-2 items-center',
                        children: left,
                    }),
                    centerWidget: Box({
                        className: 'gap-2 items-center justify-center',
                        children: center,
                    }),
                    endWidget: Box({
                        className: 'gap-2 items-center',
                        children: right,
                    }),
                }),
            ],
        }),
    });
}
