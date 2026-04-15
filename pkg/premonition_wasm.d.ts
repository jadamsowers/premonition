/* tslint:disable */
/* eslint-disable */

export class WasmEngine {
    free(): void;
    [Symbol.dispose](): void;
    all_notes_off(): void;
    get_param(param_id: number): number;
    get_params_json(): string;
    init(sample_rate: number): void;
    /**
     * Dispatch a raw 3-byte MIDI message. Status byte, data1, data2.
     * This lets the JS Web MIDI layer forward messages verbatim.
     */
    midi_message(status: number, data1: number, data2: number): void;
    constructor();
    note_off(channel: number, note: number): void;
    note_on(channel: number, note: number, velocity: number): void;
    process(output_l: Float32Array, output_r: Float32Array): void;
    set_param(param_id: number, value: number): void;
    set_params_json(json: string): void;
}

export function create_engine(): WasmEngine;

export function get_param_count(): number;

export function get_param_name(param_id: number): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly premonition_create: () => number;
    readonly premonition_destroy: (a: number) => void;
    readonly premonition_get_param: (a: number, b: number) => number;
    readonly premonition_init: (a: number, b: number) => void;
    readonly premonition_note_off: (a: number, b: number, c: number) => void;
    readonly premonition_note_on: (a: number, b: number, c: number, d: number) => void;
    readonly premonition_panic: () => void;
    readonly premonition_process: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly premonition_set_param: (a: number, b: number, c: number) => void;
    readonly __wbg_wasmengine_free: (a: number, b: number) => void;
    readonly create_engine: () => number;
    readonly get_param_count: () => number;
    readonly get_param_name: (a: number, b: number) => void;
    readonly wasmengine_all_notes_off: (a: number) => void;
    readonly wasmengine_get_param: (a: number, b: number) => number;
    readonly wasmengine_get_params_json: (a: number, b: number) => void;
    readonly wasmengine_init: (a: number, b: number) => void;
    readonly wasmengine_midi_message: (a: number, b: number, c: number, d: number) => void;
    readonly wasmengine_note_off: (a: number, b: number, c: number) => void;
    readonly wasmengine_note_on: (a: number, b: number, c: number, d: number) => void;
    readonly wasmengine_process: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly wasmengine_set_param: (a: number, b: number, c: number) => void;
    readonly wasmengine_set_params_json: (a: number, b: number, c: number, d: number) => void;
    readonly wasmengine_new: () => number;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export2: (a: number, b: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
