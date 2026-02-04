/**
 * Control Center – Phase 3 port of Caelestia control center.
 * Nav rail + panes (Audio, Network, Power, etc.) backed by sidecar.
 */
import Window from 'resource:///com/github/Aylur/ags/widget/window.js';
import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';
import Button from 'resource:///com/github/Aylur/ags/widget/button.js';
import Variable from 'resource:///com/github/Aylur/ags/variable.js';
import { controlCenterConfig, type ControlCenterPane } from '../../config/controlCenter';
import { AudioPane } from './AudioPane';
import { NetworkPane } from './NetworkPane';
import { PowerPane } from './PowerPane';

const activePane = Variable<ControlCenterPane>(controlCenterConfig.panes[0]);

const paneLabels: Record<ControlCenterPane, string> = {
    network: 'Network',
    bluetooth: 'Bluetooth',
    audio: 'Audio',
    performance: 'Performance',
    power: 'Power',
    weather: 'Weather',
};

function NavRail() {
    return Box({
        className: 'control-center-nav bg-m3-surface-container min-w-[56px] p-2 rounded-l-xl',
        vertical: true,
        children: controlCenterConfig.panes.map((paneId) =>
            Button({
                className: `rounded-lg w-full py-2 ${activePane.value === paneId ? 'bg-m3-primary text-m3-on-primary' : 'bg-transparent text-m3-on-surface-variant hover:bg-m3-surface-container-high'}`,
                child: Label({
                    label: paneLabels[paneId] ?? paneId,
                    className: 'text-sm',
                }),
                onClicked: () => activePane.setValue(paneId),
            })
        ),
    });
}

function paneWidget(id: ControlCenterPane): ReturnType<typeof Box> {
    switch (id) {
        case 'network':
            return Box({ child: NetworkPane(), vertical: true });
        case 'audio':
            return Box({ child: AudioPane(), vertical: true });
        case 'power':
            return Box({ child: PowerPane(), vertical: true });
        default:
            return Box({
                child: Label({ label: `${paneLabels[id] ?? id} (placeholder)`, className: 'p-4' }),
                vertical: true,
            });
    }
}

function PaneContent() {
    return Box({
        className: 'control-center-panes flex-1',
        child: activePane.bind().transform((id) => paneWidget(id)),
    });
}

export function ControlCenter() {
    return Window({
        name: 'control-center',
        anchor: ['top', 'right'],
        exclusivity: 'ignore',
        visible: false,
        child: Box({
            className: 'control-center rounded-xl bg-m3-surface-container-low overflow-hidden shadow-lg',
            children: [
                NavRail(),
                Box({
                    className: 'control-center-content min-w-[320px] max-h-[70vh] overflow-y-auto',
                    child: PaneContent(),
                }),
            ],
        }),
    });
}
