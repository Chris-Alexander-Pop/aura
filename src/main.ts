import App from 'resource:///com/github/Aylur/ags/app.js';
import Window from 'resource:///com/github/Aylur/ags/widget/window.js';
import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';
import { sidecar } from './services/sidecar';

const SystemStatus = () => Box({
    className: 'bg-slate-900/80 text-white p-4 rounded-xl space-x-4 border border-slate-700',
    children: [
        Label({
            className: 'text-2xl font-black text-blue-400',
            label: sidecar.bind().transform(s => s.time)
        }),
        Label({
            className: 'text-lg font-semibold text-slate-400',
            label: sidecar.bind().transform(s => `Bat: ${s.battery}% ${s.is_charging ? '⚡' : ''}`)
        }),
        Label({
            className: 'text-lg font-semibold text-purple-400',
            label: sidecar.bind().transform(s => `WS: ${s.workspace}`)
        })
    ]
});

const Bar = Window({
    name: 'bar',
    anchor: ['top', 'left', 'right'],
    exclusivity: 'exclusive',
    child: Box({
        className: 'p-2',
        children: [SystemStatus()]
    }),
});

App.config({
    style: './style/style.css',
    windows: [Bar],
});