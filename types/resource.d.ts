// Stub declarations for AGS resource:// modules (provided at runtime by AGS)
declare module 'resource:///com/github/Aylur/ags/variable.js' {
    interface VariableInstance<T = unknown> {
        value: T;
        setValue(v: T): void;
        bind(): { transform<R>(fn: (value: T) => R): R };
    }
    function Variable<T = unknown>(initial: T): VariableInstance<T>;
    export default Variable;
}

declare module 'resource:///com/github/Aylur/ags/app.js' {
    const App: {
        configDir: string;
        config: (config: object) => void;
        toggleWindow: (name: string) => void;
    };
    export default App;
}

declare module 'resource:///com/github/Aylur/ags/widget/window.js' {
    const Window: any;
    export default Window;
}

declare module 'resource:///com/github/Aylur/ags/widget/box.js' {
    const Box: any;
    export default Box;
}

declare module 'resource:///com/github/Aylur/ags/widget/label.js' {
    const Label: any;
    export default Label;
}

declare module 'resource:///com/github/Aylur/ags/widget/button.js' {
    const Button: any;
    export default Button;
}

declare module 'resource:///com/github/Aylur/ags/widget/icon.js' {
    const Icon: any;
    export default Icon;
}

declare module 'resource:///com/github/Aylur/ags/widget/centerbox.js' {
    const CenterBox: any;
    export default CenterBox;
}

declare module 'resource:///com/github/Aylur/ags/widget/scrollable.js' {
    const Scrollable: any;
    export default Scrollable;
}

declare module 'resource:///com/github/Aylur/ags/widget/stack.js' {
    const Stack: any;
    export default Stack;
}

declare module 'resource:///com/github/Aylur/ags/widget/revealer.js' {
    const Revealer: any;
    export default Revealer;
}

declare module 'resource:///com/github/Aylur/ags/widget/eventbox.js' {
    const EventBox: any;
    export default EventBox;
}

declare module 'resource:///com/github/Aylur/ags/widget/slider.js' {
    const Slider: any;
    export default Slider;
}

declare module 'resource:///com/github/Aylur/ags/widget/switch.js' {
    const Switch: any;
    export default Switch;
}

declare module 'resource:///com/github/Aylur/ags/widget/entry.js' {
    const Entry: any;
    export default Entry;
}

declare module 'resource:///com/github/Aylur/ags/service.js' {
    const Service: {
        import(name: string): Promise<any>;
    };
    export default Service;
}
