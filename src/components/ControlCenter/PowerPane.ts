import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';
import Button from 'resource:///com/github/Aylur/ags/widget/button.js';
import Variable from 'resource:///com/github/Aylur/ags/variable.js';
import { sidecar } from '../../services/sidecar';

const batteryState = Variable({ percent: 0, charging: false, time_remaining: '' });
const profile = Variable<'performance' | 'balanced' | 'power-saver'>('balanced');

// Poll
;(async () => {
    try {
        batteryState.setValue(await sidecar.getBatteryState());
    } catch (_) {}
})();
setInterval(async () => {
    try {
        batteryState.setValue(await sidecar.getBatteryState());
    } catch (_) {}
}, 5000);

export function PowerPane() {
    return Box({
        className: 'control-pane power-pane p-4',
        vertical: true,
        children: [
            Label({
                label: 'Power',
                className: 'text-lg font-semibold text-m3-on-surface',
            }),
            Label({
                label: batteryState.bind().transform((b) => `Battery: ${b.percent}% ${b.charging ? '(charging)' : ''}`),
                className: 'text-m3-on-surface-variant',
            }),
            Box({
                className: 'gap-2 flex-wrap',
                children: (['performance', 'balanced', 'power-saver'] as const).map((p) =>
                    Button({
                        child: Label({ label: p }),
                        className: `rounded-lg ${profile.value === p ? 'bg-m3-primary text-m3-on-primary' : 'bg-m3-surface-container-high'}`,
                        onClicked: () => {
                            profile.setValue(p);
                            sidecar.setPowerProfile(p).catch(() => {});
                        },
                    })
                ),
            }),
        ],
    });
}
