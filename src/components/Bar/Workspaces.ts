import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';
import Button from 'resource:///com/github/Aylur/ags/widget/button.js';
import { barConfig } from '../../config/bar';

/** Hyprland-like workspace service (from Service.import('hyprland')). */
export interface HyprlandLike {
    active: { workspace?: { id: number }; id?: number };
    workspaces: Array<{ id: number }>;
    message: (cmd: string) => void;
    getMonitor?: (id: number) => { activeWorkspace?: { id: number } };
    bind: (prop: string) => { as: (fn: (v: unknown) => unknown[]) => unknown; transform?: (fn: (v: unknown) => unknown[]) => unknown };
}

function buildWorkspaceButtons(hyprland: HyprlandLike) {
    const shown = barConfig.workspaces.shown;
    const activeId = hyprland.active?.workspace?.id ?? hyprland.active?.id ?? 1;
    const occupied = (hyprland.workspaces ?? []).reduce(
        (acc: Record<number, boolean>, w: { id: number }) => {
            acc[w.id] = true;
            return acc;
        },
        {} as Record<number, boolean>
    );
    return Array.from({ length: shown }, (_, i) => {
        const id = i + 1;
        const isActive = activeId === id;
        return Button({
            className: `rounded-full min-w-[1.5rem] ${isActive ? 'bg-m3-primary text-m3-on-primary' : 'bg-transparent text-m3-on-surface-variant'}`,
            child: Label({
                label: occupied[id] ? '●' : '○',
                className: 'text-xs',
            }),
            onClicked: () => hyprland.message(`dispatch workspace ${id}`),
        });
    });
}

/**
 * Workspace buttons for the bar (reactive to Hyprland active/workspaces).
 */
export function Workspaces(hyprland?: HyprlandLike | null) {
    const shown = barConfig.workspaces.shown;
    if (!hyprland) {
        return Box({
            className: 'bar-workspaces gap-1',
            children: Array.from({ length: shown }, (_, i) =>
                Label({ label: `${i + 1}`, className: 'text-m3-on-surface-variant text-sm' })
            ),
        });
    }
    const bind = hyprland.bind?.('active');
    const as = bind?.as ?? bind?.transform;
    return Box({
        className: 'bar-workspaces rounded-full bg-m3-surface-container p-1 gap-0.5',
        children: as ? (as.call(bind, () => buildWorkspaceButtons(hyprland)) as unknown[]) : buildWorkspaceButtons(hyprland),
    });
}
