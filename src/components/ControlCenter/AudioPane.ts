import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';
import Slider from 'resource:///com/github/Aylur/ags/widget/slider.js';
import Button from 'resource:///com/github/Aylur/ags/widget/button.js';
import Variable from 'resource:///com/github/Aylur/ags/variable.js';
import { sidecar } from '../../services/sidecar';

const volume = Variable(50);

export function AudioPane() {
    return Box({
        className: 'control-pane audio-pane p-4',
        vertical: true,
        children: [
            Label({
                label: 'Audio',
                className: 'text-lg font-semibold text-m3-on-surface',
            }),
            Box({
                className: 'gap-2 items-center',
                vertical: true,
                children: [
                    Label({
                        label: volume.bind().transform((v) => `Volume: ${Math.round(v)}%`),
                        className: 'text-m3-on-surface-variant',
                    }),
                    Slider({
                        value: volume.value,
                        onValueChanged: ({ value }: { value: number }) => {
                            volume.setValue(value);
                            sidecar.setVolume('default', value / 100).catch(() => {});
                        },
                    }),
                    Button({
                        child: Label({ label: 'Mute' }),
                        onClicked: () => sidecar.toggleMute('default').catch(() => {}),
                        className: 'rounded-lg bg-m3-surface-container-high',
                    }),
                ],
            }),
        ],
    });
}
