import { createState, createMemo, onMount, onCleanup } from "ags"
import GObject from "gi://GObject"

export function bind<T extends GObject.Object, K extends keyof T>(obj: T, prop: K) {
    const [value, setValue] = createState(obj[prop])
    
    onMount(() => {
        const id = obj.connect(`notify::${String(prop)}`, () => {
             setValue(obj[prop])
        })
        onCleanup(() => {
            obj.disconnect(id)
        })
    })

    const binding = value as any
    binding.as = (fn: (val: T[K]) => any) => {
        return createMemo(() => fn(value()))
    }
    
    return binding
}
