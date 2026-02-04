import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';
import Switch from 'resource:///com/github/Aylur/ags/widget/switch.js';
import Button from 'resource:///com/github/Aylur/ags/widget/button.js';
import Variable from 'resource:///com/github/Aylur/ags/variable.js';
import { sidecar } from '../../services/sidecar';

const wifiEnabled = Variable(true);

export function NetworkPane() {
    return Box({
        className: 'control-pane network-pane p-4',
        vertical: true,
        children: [
            Label({
                label: 'Network',
                className: 'text-lg font-semibold text-m3-on-surface',
            }),
            Box({
                className: 'gap-2 items-center',
                vertical: true,
                children: [
                    Box({
                        className: 'gap-2 items-center',
                        children: [
                            Label({
                                label: 'Wi‑Fi',
                                className: 'text-m3-on-surface-variant',
                            }),
                            Switch({
                                state: wifiEnabled.value,
                                onStateChanged: ({ state }: { state: boolean }) => {
                                    wifiEnabled.setValue(state);
                                    sidecar.toggleWifi(state).catch(() => {});
                                },
                            }),
                        ],
                    }),
                    Button({
                        child: Label({ label: 'Scan networks' }),
                        onClicked: () => sidecar.scanNetworks().catch(() => {}),
                        className: 'rounded-lg bg-m3-surface-container-high',
                    }),
                ],
            }),
        ],
    });
}
