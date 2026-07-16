import Gio from "gi://Gio"
import GLib from "gi://GLib"

const MAX_CRASH_DUMPS = 50

export type CrashComponent = "sidecar" | "ags" | "sidecar-exit"
export type CrashKind = "panic" | "uncaught" | "exit"

export interface CrashDump {
    ts: string
    component: CrashComponent
    kind: CrashKind
    message: string
    stack?: string
    host: string
    aura: string
    exit_status?: number
    signal?: number
}

/** Resolve crash dump directory: `AURA_CRASH_DIR` or `$XDG_DATA_HOME/aura/crashes`. */
export function crashDir(): string {
    const fromEnv = GLib.getenv("AURA_CRASH_DIR")
    if (fromEnv && fromEnv.length > 0) return fromEnv

    const xdg = GLib.getenv("XDG_DATA_HOME")
    if (xdg && xdg.length > 0) return GLib.build_filenamev([xdg, "aura", "crashes"])

    return GLib.build_filenamev([GLib.get_home_dir(), ".local", "share", "aura", "crashes"])
}

function hostname(): string {
    try {
        const file = Gio.File.new_for_path("/etc/hostname")
        const [ok, contents] = file.load_contents(null)
        if (ok) {
            const text = new TextDecoder().decode(contents).trim()
            if (text) return text
        }
    } catch {
        // fall through
    }
    return GLib.getenv("HOSTNAME") || "unknown"
}

function shortId(): string {
    return Math.floor(Math.random() * 0xffffffff)
        .toString(16)
        .padStart(8, "0")
}

function sanitizeComponent(component: string): string {
    return component.replace(/[^a-zA-Z0-9_-]/g, "_")
}

function ensureDir(path: string): void {
    GLib.mkdir_with_parents(path, 0o755)
}

function listJsonCrashes(dir: string): string[] {
    const file = Gio.File.new_for_path(dir)
    let enumerator: Gio.FileEnumerator | null = null
    try {
        enumerator = file.enumerate_children(
            "standard::name,time::modified",
            Gio.FileQueryInfoFlags.NONE,
            null
        )
    } catch {
        return []
    }

    const entries: { path: string; mtime: number }[] = []
    while (true) {
        const info = enumerator.next_file(null)
        if (!info) break
        const name = info.get_name()
        if (!name.toLowerCase().endsWith(".json")) continue
        const mtime = info.get_attribute_uint64("time::modified")
        entries.push({ path: GLib.build_filenamev([dir, name]), mtime })
    }
    enumerator.close(null)

    entries.sort((a, b) => a.mtime - b.mtime)
    return entries.map((e) => e.path)
}

function pruneCrashDumps(dir: string, keep: number): void {
    const paths = listJsonCrashes(dir)
    if (paths.length <= keep) return
    const removeCount = paths.length - keep
    for (let i = 0; i < removeCount; i++) {
        try {
            Gio.File.new_for_path(paths[i]).delete(null)
        } catch {
            // best-effort
        }
    }
}

function dumpToJson(dump: CrashDump): string {
    const obj: Record<string, unknown> = {
        ts: dump.ts,
        component: dump.component,
        kind: dump.kind,
        message: dump.message,
        host: dump.host,
        aura: dump.aura,
    }
    if (dump.stack) obj.stack = dump.stack
    if (dump.exit_status !== undefined) obj.exit_status = dump.exit_status
    if (dump.signal !== undefined) obj.signal = dump.signal
    return JSON.stringify(obj, null, 2) + "\n"
}

/** Write a local crash dump. Returns path or null on failure. */
export function writeCrashDump(
    partial: Omit<CrashDump, "ts" | "host" | "aura"> & {
        ts?: string
        host?: string
        aura?: string
    }
): string | null {
    try {
        const dump: CrashDump = {
            ts: partial.ts ?? new Date().toISOString(),
            component: partial.component,
            kind: partial.kind,
            message: partial.message,
            stack: partial.stack,
            host: partial.host ?? hostname(),
            aura: partial.aura ?? "ags-shell",
            exit_status: partial.exit_status,
            signal: partial.signal,
        }

        const dir = crashDir()
        ensureDir(dir)

        const tsSlug = dump.ts.replace(/[^a-zA-Z0-9]/g, "-")
        const filename = `${tsSlug}_${sanitizeComponent(dump.component)}_${shortId()}.json`
        const path = GLib.build_filenamev([dir, filename])
        const file = Gio.File.new_for_path(path)
        const bytes = new TextEncoder().encode(dumpToJson(dump))
        file.replace_contents(bytes, null, false, Gio.FileCreateFlags.REPLACE_DESTINATION, null)
        pruneCrashDumps(dir, MAX_CRASH_DUMPS)
        return path
    } catch (e) {
        console.error("aura: failed to write crash dump:", e)
        return null
    }
}

function errorMessage(err: unknown): string {
    if (err instanceof Error) return err.message || String(err)
    if (typeof err === "string") return err
    try {
        return JSON.stringify(err)
    } catch {
        return String(err)
    }
}

function errorStack(err: unknown): string | undefined {
    if (err instanceof Error && err.stack) return err.stack
    return undefined
}

/** Record an AGS/JS uncaught or handled fatal. */
export function recordUncaught(err: unknown, context?: string): void {
    const message = context
        ? `${context}: ${errorMessage(err)}`
        : errorMessage(err)
    writeCrashDump({
        component: "ags",
        kind: "uncaught",
        message,
        stack: errorStack(err),
    })
}

/** Record unexpected sidecar process exit. */
export function recordSidecarExit(opts: {
    message?: string
    exit_status?: number
    signal?: number
}): void {
    writeCrashDump({
        component: "sidecar-exit",
        kind: "exit",
        message: opts.message ?? "sidecar process exited unexpectedly",
        exit_status: opts.exit_status,
        signal: opts.signal,
    })
}

/**
 * Best-effort global handlers for uncaught errors in GJS.
 * GJS does not expose Node-style `uncaughtException`; we hook what we can.
 */
export function installCrashHandlers(): void {
    try {
        const PromiseAny = Promise as unknown as {
            _setUnhandledRejectionCallback?: (
                cb: (promise: unknown, reason: unknown) => void
            ) => void
        }
        if (typeof PromiseAny._setUnhandledRejectionCallback === "function") {
            PromiseAny._setUnhandledRejectionCallback((_promise, reason) => {
                console.error("Unhandled promise rejection:", reason)
                recordUncaught(reason, "unhandledrejection")
            })
        }
    } catch {
        // ignore — hook not available
    }
}
