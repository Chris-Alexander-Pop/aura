// Explicit module declarations so IDEs resolve gi:// without path mapping

declare module 'gi://GLib' {
    namespace GLib {
        const PRIORITY_DEFAULT: number;
        enum FileTest {
            EXISTS = 1,
        }
        function file_test(filename: string, test: GLib.FileTest): boolean;
        function source_remove(tag: number): boolean;
        function timeout_add(priority: number, interval: number, callback: () => boolean): number;
        function timeout_add_seconds(priority: number, interval: number, callback: () => boolean): number;
        class Bytes {
            constructor(data: Uint8Array | number[]);
            get_data(): Uint8Array;
        }
    }
    export default GLib;
}

declare module 'gi://Gio' {
    namespace Gio {
        enum SubprocessFlags {
            NONE = 0,
            STDIN_PIPE = 1,
            STDOUT_PIPE = 2,
            STDERR_PIPE = 4,
            STDERR_SILENCE = 32,
        }
        class Subprocess {
            static new(argv: string[], flags: Gio.SubprocessFlags): Gio.Subprocess;
            get_stdout_pipe(): unknown;
            get_stdin_pipe(): unknown;
            get_if_exited(): boolean;
            send_signal(signal: number): void;
            force_exit(): void;
            wait_async(cancellable: null, callback: (source: unknown, result: unknown) => void): void;
            wait_finish(result: unknown): boolean;
        }
        class DataInputStream {
            constructor(options: { base_stream: unknown; close_base_stream?: boolean });
            read_line_async(priority: number, cancellable: null): Promise<[Uint8Array | null, number]>;
            close(cancellable: null): void;
        }
        class DataOutputStream {
            constructor(options: { base_stream: unknown; close_base_stream?: boolean });
            put_string(str: string, cancellable: null): boolean;
            write_bytes(bytes: unknown, cancellable: null): boolean;
            close(cancellable: null): void;
        }
    }
    export default Gio;
}
