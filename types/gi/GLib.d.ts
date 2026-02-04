declare namespace GLib {
    const PRIORITY_DEFAULT: number;

    enum FileTest {
        EXISTS = 1,
    }

    function file_test(filename: string, test: FileTest): boolean;
    function source_remove(tag: number): boolean;
    function timeout_add(priority: number, interval: number, callback: () => boolean): number;
    function timeout_add_seconds(priority: number, interval: number, callback: () => boolean): number;

    class Bytes {
        constructor(data: Uint8Array | number[]);
        get_data(): Uint8Array;
    }
}

export default GLib;
