import App from 'resource:///com/github/Aylur/ags/app.js';
import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';
import Button from 'resource:///com/github/Aylur/ags/widget/button.js';

export function LauncherButton() {
    return Button({
        className: 'bar-launcher rounded p-1.5 text-m3-on-surface-variant hover:bg-m3-surface-container-high',
        child: Box({
            children: [
                Label({ label: '⌘', className: 'text-sm' }),
            ],
        }),
        onClicked: () => App.toggleWindow('app-launcher'),
    });
}
