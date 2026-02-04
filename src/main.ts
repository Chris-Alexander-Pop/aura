import App from 'resource:///com/github/Aylur/ags/app.js';
import Window from 'resource:///com/github/Aylur/ags/widget/window.js';
import Box from 'resource:///com/github/Aylur/ags/widget/box.js';
import Label from 'resource:///com/github/Aylur/ags/widget/label.js';

const HelloWorld = () => Box({
    className: 'bg-slate-900 text-white p-4 rounded-xl',
    children: [
        Label({
            className: 'text-xl font-bold',
            label: 'Welcome to your new Rust + AGS Setup!'
        })
    ]
});

const Bar = Window({
    name: 'bar',
    anchor: ['top', 'left', 'right'],
    child: HelloWorld(),
});

App.config({
    style: './style/style.css',
    windows: [Bar],
});
