import App from 'resource:///com/github/Aylur/ags/app.js';
import Variable from 'resource:///com/github/Aylur/ags/variable.js';
import Service from 'resource:///com/github/Aylur/ags/service.js';
import { sidecar } from './services/sidecar';
import { Bar } from './components/Bar';
import { ControlCenter } from './components/ControlCenter';
import { NotificationPopups } from './components/Notifications/NotificationPopups';
import { AppLauncher } from './components/Launcher/AppLauncher';

// System state (used by Bar/Power and Control Center panes)
const systemStats = Variable({ cpu: 0, ram: 0, temp: 0 });
const batteryState = Variable({ percent: 0, charging: false, time_remaining: '' });

const updateSystemStats = async () => {
    try {
        systemStats.setValue(await sidecar.getStats());
    } catch (e) {
        console.error('Failed to get system stats:', e);
    }
};
const updateBatteryState = async () => {
    try {
        batteryState.setValue(await sidecar.getBatteryState());
    } catch (e) {
        console.error('Failed to get battery state:', e);
    }
};

updateSystemStats();
updateBatteryState();
setInterval(updateSystemStats, 2000);
setInterval(updateBatteryState, 5000);

sidecar.onNotification('System.StatsUpdate', (n) => n.params && systemStats.setValue(n.params));
sidecar.onNotification('Power.BatteryChanged', (n) =>
    n.params && batteryState.setValue({ ...batteryState.value, ...n.params })
);

// Full behavior: load AGS services and register all Phase 3 windows
async function start() {
    const [hyprland, systemTray, notifications, applications] = await Promise.all([
        Service.import('hyprland').catch(() => null),
        Service.import('systemtray').catch(() => null),
        Service.import('notifications').catch(() => null),
        Service.import('applications').catch(() => null),
    ]);

    App.config({
        style: './style/style.css',
        windows: [
            Bar(hyprland, systemTray),
            ControlCenter(),
            NotificationPopups(notifications),
            AppLauncher(applications),
        ],
        closeWindowDelay: {
            'control-center': 200,
            'app-launcher': 200,
            'notification-popups': 150,
        },
    });
}

start().catch((e) => {
    console.error('AGS config failed:', e);
});
