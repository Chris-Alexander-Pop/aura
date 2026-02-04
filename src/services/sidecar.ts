import Variable from 'resource:///com/github/Aylur/ags/variable.js';

const SIDECAR_PATH = '/home/chris/.config/ags/sidecar/target/debug/ags-sidecar';

export interface SystemState {
    time: string;
    battery: number;
    is_charging: boolean;
    workspace: number;
}

const initialState: SystemState = {
    time: '--:--',
    battery: 0,
    is_charging: false,
    workspace: 1
};

export const sidecar = Variable(initialState, {
    listen: [
        SIDECAR_PATH,
        (out) => {
            try {
                return JSON.parse(out) as SystemState;
            } catch (e) {
                console.error('Failed to parse sidecar output:', out);
                return initialState;
            }
        }
    ]
});
