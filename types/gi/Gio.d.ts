declare namespace Gio {
    enum SubprocessFlags {
        NONE = 0,
        STDIN_PIPE = 1,
        STDOUT_PIPE = 2,
        STDERR_PIPE = 4,
        STDERR_SILENCE = 32,
    }

    class Subprocess {
        static new(argv: string[], flags: SubprocessFlags): Subprocess;
        get_stdout_pipe(): any;
        get_stdin_pipe(): any;
        get_if_exited(): boolean;
        send_signal(signal: number): void;
        force_exit(): void;
        wait_async(cancellable: null, callback: (source: any, result: any) => void): void;
        wait_finish(result: any): boolean;
    }

    class DataInputStream {
        constructor(options: { base_stream: any; close_base_stream?: boolean });
        read_line_async(priority: number, cancellable: null): Promise<[Uint8Array | null, number]>;
        close(cancellable: null): void;
    }

    class DataOutputStream {
        constructor(options: { base_stream: any; close_base_stream?: boolean });
        put_string(str: string, cancellable: null): boolean;
        write_bytes(bytes: any, cancellable: null): boolean;
        close(cancellable: null): void;
    }
}

export default Gio;
