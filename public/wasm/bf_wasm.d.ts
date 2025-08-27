/* tslint:disable */
/* eslint-disable */
export class BrainfuckInterpreter {
  free(): void;
  constructor();
  static with_options(tape_size: number, cell_size: number, wrap: boolean, wrap_tape: boolean, optimize: boolean): BrainfuckInterpreter;
  run_program(code: string, input: Uint8Array): any;
  run_program_with_callback(code: string, input: Uint8Array, output_callback: Function): any;
  optimize_brainfuck(code: string): string;
}
export class StatefulBrainfuckInterpreter {
  free(): void;
  constructor(code: string, tape_size: number, cell_size: number, wrap: boolean, wrap_tape: boolean, optimize_flag: boolean);
  run_until_input(output_callback: Function): boolean;
  provide_input(char_code: number): void;
  get_state(): any;
  is_waiting_for_input(): boolean;
  is_finished(): boolean;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_brainfuckinterpreter_free: (a: number, b: number) => void;
  readonly __wbg_statefulbrainfuckinterpreter_free: (a: number, b: number) => void;
  readonly brainfuckinterpreter_new: () => number;
  readonly brainfuckinterpreter_with_options: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly brainfuckinterpreter_run_program: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly brainfuckinterpreter_run_program_with_callback: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly brainfuckinterpreter_optimize_brainfuck: (a: number, b: number, c: number, d: number) => void;
  readonly statefulbrainfuckinterpreter_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
  readonly statefulbrainfuckinterpreter_run_until_input: (a: number, b: number) => number;
  readonly statefulbrainfuckinterpreter_provide_input: (a: number, b: number) => void;
  readonly statefulbrainfuckinterpreter_get_state: (a: number) => number;
  readonly statefulbrainfuckinterpreter_is_waiting_for_input: (a: number) => number;
  readonly statefulbrainfuckinterpreter_is_finished: (a: number) => number;
  readonly __wbindgen_export_0: (a: number) => void;
  readonly __wbindgen_export_1: (a: number, b: number) => number;
  readonly __wbindgen_export_2: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export_3: (a: number, b: number, c: number) => void;
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
