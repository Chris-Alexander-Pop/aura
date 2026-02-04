import Variable from 'resource:///com/github/Aylur/ags/variable.js';
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';

const SIDECAR_PATH = '/home/user/.config/ags/sidecar/target/release/ags-sidecar';
const SIDECAR_PATH_DEBUG = '/home/user/.config/ags/sidecar/target/debug/ags-sidecar';

export interface JsonRpcRequest {
    jsonrpc: '2.0';
    method: string;
    params?: any;
    id?: number | string;
}

export interface JsonRpcResponse {
    jsonrpc: '2.0';
    result?: any;
    error?: {
        code: number;
        message: string;
        data?: any;
    };
    id?: number | string;
}

export interface JsonRpcNotification {
    jsonrpc: '2.0';
    method: string;
    params?: any;
}

type RequestCallback = (response: JsonRpcResponse) => void;
type NotificationHandler = (notification: JsonRpcNotification) => void;

class SidecarClient {
    private process: Gio.Subprocess | null = null;
    private stdout: Gio.DataInputStream | null = null;
    private stdin: Gio.DataOutputStream | null = null;
    private requestIdCounter = 1;
    private pendingRequests = new Map<number | string, RequestCallback>();
    private notificationHandlers = new Map<string, NotificationHandler[]>();
    private reconnectTimer: number | null = null;
    private reconnectAttempts = 0;
    private maxReconnectAttempts = 5;
    private reconnectDelay = 2000; // ms

    constructor(private sidecarPath: string = SIDECAR_PATH) {}

    async connect(): Promise<void> {
        if (this.process && this.process.get_if_exited() === false) {
            return; // Already connected
        }

        try {
            // Try release first, fallback to debug
            let path = this.sidecarPath;
            if (!GLib.file_test(path, GLib.FileTest.EXISTS)) {
                path = SIDECAR_PATH_DEBUG;
            }

            this.process = Gio.Subprocess.new(
                [path, 'server'],
                Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDIN_PIPE | Gio.SubprocessFlags.STDERR_SILENCE
            );

            this.stdout = new Gio.DataInputStream({
                base_stream: this.process.get_stdout_pipe(),
                close_base_stream: true,
            });

            this.stdin = new Gio.DataOutputStream({
                base_stream: this.process.get_stdin_pipe(),
                close_base_stream: true,
            });

            // Start reading responses
            this.readLoop();

            this.reconnectAttempts = 0;
            console.log('Sidecar connected');
        } catch (error) {
            console.error('Failed to connect to sidecar:', error);
            this.scheduleReconnect();
        }
    }

    private async readLoop(): Promise<void> {
        if (!this.stdout) return;

        try {
            while (true) {
                const [line] = await this.stdout.read_line_async(
                    GLib.PRIORITY_DEFAULT,
                    null
                );

                if (!line) break;

                const text = new TextDecoder().decode(line);
                const trimmed = text.trim();

                if (!trimmed) continue;

                try {
                    const json = JSON.parse(trimmed);

                    if (json.id !== undefined) {
                        // This is a response
                        const callback = this.pendingRequests.get(json.id);
                        if (callback) {
                            callback(json as JsonRpcResponse);
                            this.pendingRequests.delete(json.id);
                        }
                    } else {
                        // This is a notification
                        this.handleNotification(json as JsonRpcNotification);
                    }
                } catch (e) {
                    console.error('Failed to parse sidecar output:', trimmed, e);
                }
            }
        } catch (error) {
            console.error('Error reading from sidecar:', error);
            this.scheduleReconnect();
        }
    }

    private handleNotification(notification: JsonRpcNotification): void {
        const handlers = this.notificationHandlers.get(notification.method);
        if (handlers) {
            handlers.forEach(handler => handler(notification));
        }
    }

    private scheduleReconnect(): void {
        if (this.reconnectTimer) {
            GLib.source_remove(this.reconnectTimer);
        }

        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('Max reconnect attempts reached');
            return;
        }

        this.reconnectAttempts++;
        console.log(`Scheduling reconnect attempt ${this.reconnectAttempts}...`);

        this.reconnectTimer = GLib.timeout_add(
            GLib.PRIORITY_DEFAULT,
            this.reconnectDelay,
            () => {
                this.connect();
                return false;
            }
        );
    }

    async request(method: string, params?: any): Promise<any> {
        if (!this.process || !this.stdin) {
            await this.connect();
            // Wait a bit for connection to establish
            await new Promise(resolve => setTimeout(resolve, 100));
        }

        if (!this.stdin) {
            throw new Error('Sidecar not connected');
        }

        const id = this.requestIdCounter++;
        const request: JsonRpcRequest = {
            jsonrpc: '2.0',
            method,
            params,
            id,
        };

        return new Promise((resolve, reject) => {
            this.pendingRequests.set(id, (response: JsonRpcResponse) => {
                if (response.error) {
                    reject(new Error(response.error.message || 'RPC error'));
                } else {
                    resolve(response.result);
                }
            });

            try {
                const json = JSON.stringify(request) + '\n';
                const bytes = new TextEncoder().encode(json);
                this.stdin!.write_bytes(
                    new GLib.Bytes(bytes),
                    null
                );
            } catch (error) {
                this.pendingRequests.delete(id);
                reject(error);
            }

            // Timeout after 30 seconds
            GLib.timeout_add_seconds(
                GLib.PRIORITY_DEFAULT,
                30,
                () => {
                    if (this.pendingRequests.has(id)) {
                        this.pendingRequests.delete(id);
                        reject(new Error('Request timeout'));
                    }
                    return false;
                }
            );
        });
    }

    onNotification(method: string, handler: NotificationHandler): void {
        if (!this.notificationHandlers.has(method)) {
            this.notificationHandlers.set(method, []);
        }
        this.notificationHandlers.get(method)!.push(handler);
    }

    offNotification(method: string, handler: NotificationHandler): void {
        const handlers = this.notificationHandlers.get(method);
        if (handlers) {
            const index = handlers.indexOf(handler);
            if (index !== -1) {
                handlers.splice(index, 1);
            }
        }
    }

    disconnect(): void {
        if (this.reconnectTimer) {
            GLib.source_remove(this.reconnectTimer);
            this.reconnectTimer = null;
        }

        if (this.stdin) {
            try {
                this.stdin.close(null);
            } catch (e) {
                // Ignore
            }
            this.stdin = null;
        }

        if (this.stdout) {
            try {
                this.stdout.close(null);
            } catch (e) {
                // Ignore
            }
            this.stdout = null;
        }

        if (this.process) {
            try {
                this.process.force_exit();
            } catch (e) {
                // Ignore
            }
            this.process = null;
        }

        this.pendingRequests.clear();
    }
}

// Singleton instance
export const sidecarClient = new SidecarClient();

// Auto-connect on import
sidecarClient.connect().catch(err => {
    console.error('Failed to auto-connect sidecar:', err);
});

// Export convenience methods for common operations
export const sidecar = {
    // System
    async getStats() {
        return sidecarClient.request('System.GetStats');
    },

    // Power
    async getBatteryState() {
        return sidecarClient.request('Power.GetBatteryState');
    },

    async setPowerProfile(profile: 'performance' | 'balanced' | 'power-saver') {
        return sidecarClient.request('Power.SetProfile', { profile });
    },

    // Network
    async toggleWifi(enabled: boolean) {
        return sidecarClient.request('Network.ToggleWifi', { enabled });
    },

    async scanNetworks() {
        return sidecarClient.request('Network.ScanNetworks');
    },

    // VPN
    async connectVpn(profileId: string, auth?: { user?: string; pass?: string; mfa?: string }) {
        return sidecarClient.request('Vpn.Connect', { profile_id: profileId, auth });
    },

    async disconnectVpn() {
        return sidecarClient.request('Vpn.Disconnect');
    },

    async getVpnProfiles() {
        return sidecarClient.request('Vpn.GetProfiles');
    },

    // Audio
    async setVolume(stream: string, percent: number) {
        return sidecarClient.request('Audio.SetVolume', { stream, percent });
    },

    async toggleMute(stream: string) {
        return sidecarClient.request('Audio.ToggleMute', { stream });
    },

    // Generic request method
    request: (method: string, params?: any) => sidecarClient.request(method, params),

    // Notification handling
    onNotification: (method: string, handler: NotificationHandler) => 
        sidecarClient.onNotification(method, handler),
    
    offNotification: (method: string, handler: NotificationHandler) =>
        sidecarClient.offNotification(method, handler),
};
