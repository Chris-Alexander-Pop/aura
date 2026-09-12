// @ts-ignore
import GObject from 'gi://GObject'
import Gio from 'gi://Gio'
import GLib from 'gi://GLib'

const HYPRLAND_SIGNATURE = GLib.getenv("HYPRLAND_INSTANCE_SIGNATURE")
const XDG_RUNTIME_DIR = GLib.getenv("XDG_RUNTIME_DIR")

export interface Workspace {
    id: number
    name: string
}

class HyprlandService extends GObject.Object {
    static {
        GObject.registerClass({
            GTypeName: 'HyprlandService',
            Signals: {
                'workspaces-changed': {},
                'focused-workspace-changed': { param_types: [GObject.TYPE_INT] },
            }
        }, this)
    }

    private _workspaces: Workspace[] = []
    private _focusedWorkspaceId: number = 1
    private _eventStream: Gio.DataInputStream | null = null

    constructor() {
        super()
        this._initialize()
    }

    get workspaces() {
        return this._workspaces
    }

    get focusedWorkspaceId() {
        return this._focusedWorkspaceId
    }

    private async _initialize() {
        if (!HYPRLAND_SIGNATURE || !XDG_RUNTIME_DIR) {
            console.error("Hyprland environment variables missing")
            return
        }

        await this._fetchState()
        this._connectEventSocket()
    }

    private async _fetchState() {
        try {
            const [res, out] = GLib.spawn_command_line_sync("hyprctl -j workspaces")
            if (res && out) {
                const decoder = new TextDecoder()
                const json = decoder.decode(out)
                this._workspaces = JSON.parse(json).map((w: any) => ({
                    id: Number(w.id ?? w.address ?? w.name),
                    name: w.name ?? String(w.address ?? w.id ?? ""),
                })).filter((w: Workspace) => Number.isFinite(w.id) && w.id > 0)
                this.emit('workspaces-changed')
            }

             const [res2, out2] = GLib.spawn_command_line_sync("hyprctl -j monitors")
            if (res2 && out2) {
                 const decoder = new TextDecoder()
                 const json = decoder.decode(out2)
                 const monitors = JSON.parse(json)
                 const focused = monitors.find((m: any) => m.focused)
                 if (focused) {
                     const aw = focused.activeWorkspace ?? focused.active_workspace ?? {}
                     this._focusedWorkspaceId = Number(aw.id ?? aw.address ?? aw.name)
                     this.emit('focused-workspace-changed', this._focusedWorkspaceId)
                 }
            }

        } catch (e) {
            console.error("Failed to fetch initial Hyprland state", e)
        }
    }

    private _connectEventSocket() {
        const socketPath = `${XDG_RUNTIME_DIR}/hypr/${HYPRLAND_SIGNATURE}/.socket2.sock`
        const client = new Gio.SocketClient()
        const conn = client.connect(new Gio.UnixSocketAddress({ path: socketPath }), null)
        
        const stream = new Gio.DataInputStream({
            base_stream: conn.get_input_stream(),
            close_base_stream: true
        })
        
        this._readEvents(stream)
    }

    private _readEvents(stream: Gio.DataInputStream) {
        stream.read_line_async(GLib.PRIORITY_DEFAULT, null, (obj, res) => {
            try {
                const [line] = stream.read_line_finish(res)
                if (line) {
                    const decoder = new TextDecoder()
                    const eventStr = decoder.decode(line)
                    this._handleEvent(eventStr)
                }
                this._readEvents(stream) // Loop
            } catch (e) {
                console.error("Error reading Hyprland socket", e)
            }
        })
    }

    private _handleEvent(eventStr: string) {
        const [event, params] = eventStr.split('>>')
        
        if (event === 'workspace') {
             this._focusedWorkspaceId = parseInt(params)
             this.emit('focused-workspace-changed', this._focusedWorkspaceId)
        } else if (event === 'createworkspace' || event === 'destroyworkspace') {
            this._fetchState() // Refresh workspaces list
        }
    }
    
    public messageAsync(cmd: string) {
        // Simple fire-and-forget command execution for now
        return GLib.spawn_command_line_async(`hyprctl dispatch ${cmd}`)
    }
}

const service = new HyprlandService()
export default service
