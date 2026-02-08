declare module "gi://AstalTray" {
    import GObject from "gi://GObject"
    export default class Tray extends GObject.Object {
        static get_default(): Tray
        readonly items: any[]
    }
}

declare module "gi://AstalMpris" {
    import GObject from "gi://GObject"
    export default class Mpris extends GObject.Object {
        static get_default(): Mpris
        readonly players: any[]
    }
}

declare module "gi://AstalApps" {
    import GObject from "gi://GObject"
    export default class Apps extends GObject.Object {
        constructor()
        readonly list: Application[]
        get_list(): Application[]
        fuzzy_query(query: string): Application[]
    }

    export class Application extends GObject.Object {
        app: any
        name: string
        description: string
        icon_name: string
        launch(): void
    }
}

declare module "gi://AstalNotifd" {
    import GObject from "gi://GObject"

    export default class Notifd extends GObject.Object {
        static get_default(): Notifd
        readonly notifications: Notification[]
        readonly dont_disturb: boolean
        set_dont_disturb(value: boolean): void
    }

    export class Notification extends GObject.Object {
        readonly id: number
        readonly app_name: string
        readonly app_icon: string
        readonly summary: string
        readonly body: string
        readonly actions: Action[]
        readonly urgency: Urgency
        readonly time: number
        readonly image: string
        dismiss(): void
        invoke(actionId: string): void
    }

    export interface Action {
        id: string
        label: string
    }

    export enum Urgency {
        LOW = 0,
        NORMAL = 1,
        CRITICAL = 2,
    }
}

declare module "gi://Astal" {
    export enum WindowAnchor {
        TOP = 1,
        BOTTOM = 2,
        LEFT = 4,
        RIGHT = 8,
    }
    export enum Keymode {
        ON_DEMAND = 1,
        EXCLUSIVE = 2,
    }
    export enum Exclusivity {
        EXCLUSIVE = 1,
        NORMAL = 0,
    }
}
