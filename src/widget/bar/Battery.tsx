import { createState, onMount, createMemo } from "ags"
import sidecar from "../../lib/sidecar"

export default function Battery() {
    const [percent, setPercent] = createState(0)
    const [charging, setCharging] = createState(false)

    onMount(() => {
        // @ts-ignore
        const id = sidecar.connect('battery-state', (_, state) => {
            setPercent(state.percent)
            setCharging(state.charging)
        })
    })

    const labelText = createMemo(() => `BAT: ${percent()}%`)
    const chargingIcon = createMemo(() => charging() ? '⚡' : '')

    return <box class="Battery">
        <label label={labelText} />
        <label label={chargingIcon} />
    </box>
}
