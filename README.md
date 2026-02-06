# AGS Config

A custom configuration for Aylur's GTK Shell (AGS), powered by TypeScript, TailwindCSS, and a Rust sidecar.

## Features

- **TypeScript** based for type safety.
- **TailwindCSS** for styling.
- **Rust Sidecar** for high-performance backend logic.
- **Bun** as the package manager and runner.

## Prerequisites

- [AGS](https://github.com/Aylur/ags)
- [Bun](https://bun.sh)
- [Rust](https://www.rust-lang.org) (for the sidecar)

## Development

Use the `aura` script for a complete development environment (Rust watch + CSS watch + AGS):

```bash
./aura
```

Or run frontend only:

```bash
bun run watch
```
