export { RippleAssembler } from './assembler.ts';
export { InstructionEncoder } from './encoder.ts';
export { Parser } from './parser.ts';
export { MacroFormatter } from './macro-formatter.ts';
export { Disassembler, disassembler } from './disassembler.ts';
export type { DisassembledInstruction } from './disassembler.ts';
export * from './types.ts';

// Create a singleton instance for the IDE
import { RippleAssembler } from './assembler.ts';
import { MacroFormatter } from './macro-formatter.ts';
import type { Instruction, AssemblerOptions } from './types.ts';

export const ripplerAssembler = new RippleAssembler();

// Create assembler with custom options
export function createAssembler(options?: AssemblerOptions): RippleAssembler {
    return new RippleAssembler(options);
}

// Export formatMacro function
const formatter = new MacroFormatter();
export function formatMacro(instructions: Instruction[], memoryData: number[]): string {
    return formatter.formatFullProgram(instructions, memoryData);
}