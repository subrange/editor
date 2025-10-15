// Export the new Rust WASM assembler
// export {
//   RippleAssembler,
//   createAssembler,
//   initAssembler,
// } from './assembler.ts';

// Export other utilities that are still needed
export { MacroFormatter } from './macro-formatter.ts';
export { Disassembler, disassembler } from './disassembler.ts';
export type { DisassembledInstruction } from './disassembler.ts';
export * from './types.ts';

// Export formatMacro function for compatibility
import { MacroFormatter } from './macro-formatter.ts';
import type { Instruction } from './types.ts';

const formatter = new MacroFormatter();
export function formatMacro(
  instructions: Instruction[],
  memoryData: number[],
): string {
  return formatter.formatFullProgram(instructions, memoryData);
}
