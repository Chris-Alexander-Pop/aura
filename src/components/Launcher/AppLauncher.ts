/**
 * App Launcher – Phase 3 port of Caelestia launcher.
 * Search entry + filtered app list. Uses AGS Applications when available.
 */
import Window from 'resource:///com/github/Aylur/ags/widget/window.js';
import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';
import Button from 'resource:///com/github/Aylur/ags/widget/button.js';
import Entry from 'resource:///com/github/Aylur/ags/widget/entry.js';
import Scrollable from 'resource:///com/github/Aylur/ags/widget/scrollable.js';
import Variable from 'resource:///com/github/Aylur/ags/variable.js';
import { launcherConfig } from '../../config/launcher';

/** Application entry from Applications.list. */
export interface AppItemLike {
    name: string;
    icon_name?: string;
    desktop?: string;
    launch: () => void;
}

/** Applications service from Service.import('applications'). */
export interface ApplicationsLike {
    list: AppItemLike[];
    bind: (prop: string) => { as: (fn: (list: AppItemLike[]) => unknown[]) => unknown };
}

const searchQuery = Variable('');

function filterApps(list: AppItemLike[], q: string, max: number): AppItemLike[] {
    const lower = q.trim().toLowerCase();
    return lower
        ? list.filter((app) => app.name.toLowerCase().includes(lower)).slice(0, max)
        : list.slice(0, max);
}

/**
 * App launcher window: search + app list (reactive to applications list and search).
 */
export function AppLauncher(applications?: ApplicationsLike | null) {
    const maxShown = launcherConfig.maxShown;
    const appList = Variable<AppItemLike[]>(applications?.list ?? []);
    const filteredList = Variable<AppItemLike[]>(filterApps(applications?.list ?? [], '', maxShown));

    function updateFilter() {
        filteredList.setValue(filterApps(appList.value, searchQuery.value, maxShown));
    }

    applications?.bind?.('list')?.as?.((list: AppItemLike[]) => {
        appList.setValue(list ?? []);
        updateFilter();
        return [];
    });

    return Window({
        name: 'app-launcher',
        anchor: ['top'],
        exclusivity: 'ignore',
        visible: false,
        child: Box({
            className: 'app-launcher rounded-xl bg-m3-surface-container p-4 min-w-[320px] max-h-[70vh] border border-m3-outline/30',
            vertical: true,
            children: [
                Box({
                    child: Entry({
                        placeholderText: 'Search apps...',
                        text: searchQuery.value,
                        onAccept: () => {},
                        onChange: ({ text }: { text: string }) => {
                            searchQuery.setValue(text);
                            updateFilter();
                        },
                        className: 'launcher-entry rounded-lg bg-m3-surface-container-high p-2',
                    }),
                }),
                Scrollable({
                    className: 'launcher-list mt-2',
                    child: Box({
                        className: 'gap-1',
                        vertical: true,
                        children: filteredList.bind().transform((apps) =>
                            apps.map((app) =>
                                Button({
                                    className: 'launcher-item rounded-lg p-2 text-left hover:bg-m3-surface-container-high',
                                    child: Label({
                                        label: app.name,
                                        className: 'text-m3-on-surface',
                                        xalign: 0,
                                    }),
                                    onClicked: () => app.launch(),
                                })
                            )
                        ),
                    }),
                }),
            ],
        }),
    });
}
