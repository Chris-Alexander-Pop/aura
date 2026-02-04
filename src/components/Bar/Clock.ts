import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';
import Variable from 'resource:///com/github/Aylur/ags/variable.js';
import { barConfig } from '../../config/bar';

function formatTime(twelveHour: boolean): string {
    const d = new Date();
    if (twelveHour) {
        return d.toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit', hour12: true });
    }
    return d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', hour12: false });
}

const useTwelve = barConfig.clock.useTwelveHour ?? false;
const timeLabel = Variable(formatTime(useTwelve));
setInterval(() => timeLabel.setValue(formatTime(useTwelve)), 1000);

export function Clock() {
    return Box({
        className: 'bar-clock gap-1 text-m3-tertiary',
        vertical: true,
        children: [
            barConfig.clock.showIcon
                ? Box({
                      child: Label({
                          label: '📅',
                          className: 'text-xs',
                      }),
                  })
                : null,
            Label({
                className: 'font-mono text-sm',
                label: timeLabel.bind().transform((t) => t),
            }),
        ].filter(Boolean),
    });
}
