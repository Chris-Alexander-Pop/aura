import sidecar from "../lib/sidecar";
// @ts-ignore
import GObject from "gi://GObject";

class NetworkService extends GObject.Object {
    static {
        GObject.registerClass({
            GTypeName: 'NetworkService',
            Signals: {}
        }, this);
    }

    async getStatus() {
        return await sidecar.getNetworkStatus();
    }

    async scan() {
        return await sidecar.scanNetworks();
    }

    // Connect via sidecar (NM / keyring first)
    async connectToNetwork(ssid: string, password?: string) {
        // For now, assume simple connection or use sidecar's future connect method
        // sidecar.connectNetwork(ssid, password)
        console.log(`Connecting to ${ssid}...`);
        // TODO: Implement connection logic in Sidecar or here via nmcli if needed for now
    }
}

const network = new NetworkService();
export default network;
