/**
 * Notification popups – reactive to Notifications service.
 */
import Window from 'resource:///com/github/Aylur/ags/widget/window.js';
import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';
import Button from 'resource:///com/github/Aylur/ags/widget/button.js';

type PopupItem = {
    summary: string;
    body: string;
    app_name?: string;
    urgency?: string;
    time: number;
    image?: string;
    app_icon?: string;
    actions?: Array<{ id: string; label: string }>;
    close: () => void;
    invoke: (id: string) => void;
};

export interface NotificationsLike {
    popups: PopupItem[];
    bind: (prop: string) => { as: (fn: (popups: PopupItem[]) => unknown[]) => unknown };
}

function popupCards(popups: PopupItem[]) {
    return popups.map((popup) =>
        Box({
            className: 'notification-card rounded-xl bg-m3-surface-container p-3 min-w-[280px] max-w-[360px] border border-m3-outline/30',
            vertical: true,
            children: [
                Box({
                    children: [
                        Label({
                            label: popup.app_name ?? 'Notification',
                            className: 'text-m3-on-surface-variant text-xs',
                        }),
                        Label({
                            label: popup.summary,
                            className: 'text-m3-on-surface font-medium',
                            xalign: 0,
                        }),
                    ],
                    vertical: true,
                }),
                popup.body
                    ? Label({
                          label: popup.body,
                          className: 'text-m3-on-surface-variant text-sm',
                          wrap: true,
                          maxWidthChars: 40,
                      })
                    : null,
                Box({
                    className: 'gap-2 mt-2',
                    children: [
                        Button({
                            child: Label({ label: 'Close' }),
                            className: 'rounded-lg bg-m3-surface-container-high text-m3-on-surface',
                            onClicked: () => popup.close(),
                        }),
                    ],
                }),
            ].filter(Boolean),
        })
    );
}

export function NotificationPopups(notifications?: NotificationsLike | null) {
    if (!notifications) {
        return Window({
            name: 'notification-popups',
            anchor: ['top', 'right'],
            visible: false,
            child: Box(),
        });
    }
    const bind = notifications.bind?.('popups');
    const as = bind?.as;
    return Window({
        name: 'notification-popups',
        anchor: ['top', 'right'],
        child: Box({
            className: 'notification-popups gap-2 p-2',
            vertical: true,
            children: as
                ? (as.call(bind, (popups: PopupItem[]) => popupCards(popups ?? [])) as unknown[])
                : popupCards(notifications.popups ?? []),
        }),
    });
}
