import App from 'resource:///com/github/Aylur/ags/app.js';
import Window from 'resource:///com/github/Aylur/ags/widget/window.js';
import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';
import Variable from 'resource:///com/github/Aylur/ags/variable.js';
import { sidecar } from './services/sidecar';

// System state variables
const systemStats = Variable({ cpu: 0, ram: 0, temp: 0 });
const batteryState = Variable({ percent: 0, charging: false, time_remaining: '' });

// Poll system stats every 2 seconds
const updateSystemStats = async () => {
    try {
        const stats = await sidecar.getStats();
        systemStats.setValue(stats);
    } catch (error) {
        console.error('Failed to get system stats:', error);
    }
};

const updateBatteryState = async () => {
    try {
        const battery = await sidecar.getBatteryState();
        batteryState.setValue(battery);
    } catch (error) {
        console.error('Failed to get battery state:', error);
    }
};

// Initial updates
updateSystemStats();
updateBatteryState();

// Set up polling
setInterval(updateSystemStats, 2000);
setInterval(updateBatteryState, 5000);

// Listen for notifications
sidecar.onNotification('System.StatsUpdate', (notification) => {
    if (notification.params) {
        systemStats.setValue(notification.params);
    }
});

sidecar.onNotification('Power.BatteryChanged', (notification) => {
    if (notification.params) {
        batteryState.setValue({ ...batteryState.value, ...notification.params });
    }
});

const SystemStatus = () => Box({
    className: 'bg-m3-surface-container/80 text-m3-on-surface p-4 rounded-xl space-x-4 border border-m3-outline/35',
    children: [
        Label({
            className: 'text-2xl font-black text-m3-primary',
            label: systemStats.bind().transform(s => `CPU: ${s.cpu.toFixed(1)}%`)
        }),
        Label({
            className: 'text-lg font-semibold text-m3-on-surface-variant',
            label: systemStats.bind().transform(s => `RAM: ${s.ram.toFixed(1)}%`)
        }),
        Label({
            className: 'text-lg font-semibold text-m3-secondary',
            label: batteryState.bind().transform(b => `Bat: ${b.percent}% ${b.charging ? '⚡' : ''}`)
        })
    ]
});

const Bar = Window({
    name: 'bar',
    anchor: ['top', 'left', 'right'],
    exclusivity: 'exclusive',
    child: Box({
        className: 'p-2 bg-m3-surface/transparency-base',
        children: [SystemStatus()]
    }),
});

App.config({
    style: './style/style.css',
    windows: [Bar],
});