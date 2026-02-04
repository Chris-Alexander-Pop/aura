import App from 'resource:///com/github/Aylur/ags/app.js';
import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';
import Button from 'resource:///com/github/Aylur/ags/widget/button.js';
import Variable from 'resource:///com/github/Aylur/ags/variable.js';
import { sidecar } from '../../services/sidecar';

const batteryState = Variable({ percent: 0, charging: false, time_remaining: '' });

// Poll battery
(async () => {
    try {
        const b = await sidecar.getBatteryState();
        batteryState.setValue(b);
    } catch (_) {}
})();
setInterval(async () => {
    try {
        const b = await sidecar.getBatteryState();
        batteryState.setValue(b);
    } catch (_) {}
}, 5000);

sidecar.onNotification('Power.BatteryChanged', (n) => {
    if (n.params) batteryState.setValue({ ...batteryState.value, ...n.params });
});

/**
 * Power/battery indicator and optional control center trigger.
 */
export function Power() {
    return Box({
        className: 'bar-power gap-1 items-center',
        children: [
            Label({
                className: 'text-m3-on-surface-variant text-sm',
                label: batteryState.bind().transform((b) => `Bat: ${b.percent}% ${b.charging ? '⚡' : ''}`),
            }),
            Button({
                className: 'rounded p-1 text-m3-on-surface-variant hover:bg-m3-surface-container-high',
                child: Label({ label: '⚙', className: 'text-xs' }),
                onClicked: () => App.toggleWindow('control-center'),
            }),
        ],
    });
}
