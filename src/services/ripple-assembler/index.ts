export { RippleAssembler } from './assembler.ts';
export { InstructionEncoder } from './encoder.ts';
export { Parser } from './parser.ts';
export { MacroFormatter } from './macro-formatter.ts';
export * from './types.ts';

// Create a singleton instance for the IDE
import { RippleAssembler } from './assembler.ts';
import { MacroFormatter } from './macro-formatter.ts';
import type { Instruction } from './types.ts';

export const ripplerAssembler = new RippleAssembler();

// Export formatMacro function
const formatter = new MacroFormatter();
export function formatMacro(instructions: Instruction[], memoryData: number[]): string {
    return formatter.formatFullProgram(instructions, memoryData);
}