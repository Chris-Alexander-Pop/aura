import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Button from 'resource:///com/github/Aylur/ags/widget/button.js';
import Icon from 'resource:///com/github/Aylur/ags/widget/icon.js';
import { barConfig } from '../../config/bar';

type TrayItem = {
    icon: unknown;
    tooltip_markup?: string;
    activate: (e: unknown) => void;
    openMenu: (e: unknown) => void;
};

/** SystemTray from Service.import('systemtray'). */
export interface SystemTrayLike {
    items: TrayItem[];
    bind: (prop: string) => { as: (fn: (items: TrayItem[]) => unknown[]) => unknown };
}

function trayItemButtons(items: TrayItem[]) {
    return items.map((item) =>
        Button({
            child: Icon({ icon: item.icon, size: 18 }),
            tooltip: item.tooltip_markup ?? '',
            onPrimaryClick: (_: unknown, event: unknown) => item.activate(event),
            onSecondaryClick: (_: unknown, event: unknown) => item.openMenu(event),
            className: 'min-w-[1.5rem] min-h-[1.5rem] rounded',
        })
    );
}

/**
 * System tray icons (reactive via bind when available).
 */
export function Tray(systemTray?: SystemTrayLike | null) {
    if (!barConfig.entries.find((e) => e.id === 'tray' && e.enabled)) {
        return Box({ className: 'bar-tray', children: [] });
    }
    if (!systemTray) {
        return Box({ className: 'bar-tray', children: [] });
    }
    const bind = systemTray.bind?.('items');
    const as = bind?.as;
    return Box({
        className: 'bar-tray gap-1',
        children: as ? (as.call(bind, (items: TrayItem[]) => trayItemButtons(items)) as unknown[]) : trayItemButtons(systemTray.items ?? []),
    });
}
