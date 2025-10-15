import { Opcode, InstructionFormat, opcodeInfo } from './types.ts';

export type DisassembledInstruction = [
  string,
  string | null,
  string | null,
  string | null,
];

const registerNames: Record<number, string> = {
  0: 'R0', // Zero register
  1: 'PC', // Program Counter
  2: 'PCB', // Program Counter Bank
  3: 'RA', // Return Address
  4: 'RAB', // Return Address Bank
  5: 'RV0', // Return Value 0
  6: 'RV1', // Return Value 1
  7: 'A0', // Argument 0
  8: 'A1', // Argument 1
  9: 'A2', // Argument 2
  10: 'A3', // Argument 3
  11: 'X0', // Reserved/Extended 0
  12: 'X1', // Reserved/Extended 1
  13: 'X2', // Reserved/Extended 2
  14: 'X3', // Reserved/Extended 3
  15: 'T0', // Temporary 0
  16: 'T1', // Temporary 1
  17: 'T2', // Temporary 2
  18: 'T3', // Temporary 3
  19: 'T4', // Temporary 4
  20: 'T5', // Temporary 5
  21: 'T6', // Temporary 6
  22: 'T7', // Temporary 7
  23: 'S0', // Saved 0
  24: 'S1', // Saved 1
  25: 'S2', // Saved 2
  26: 'S3', // Saved 3
  27: 'SC', // Allocator Scratch
  28: 'SB', // Stack Bank
  29: 'SP', // Stack Pointer
  30: 'FP', // Frame Pointer
  31: 'GP', // Global Pointer
};

export class Disassembler {
  private getRegisterName(reg: number): string {
    return registerNames[reg] || `R${reg}`;
  }

  disassemble(
    word0: number,
    word1: number,
    word2: number,
    word3: number,
  ): DisassembledInstruction {
    const opcode = word0 as Opcode;
    const info = opcodeInfo[opcode];

    if (!info) {
      return ['UNKNOWN', `0x${word0.toString(16)}`, null, null];
    }

    const mnemonic = info.mnemonic;

    switch (info.format) {
      case InstructionFormat.R:
        // Special case for HALT (NOP with all zeros)
        if (
          opcode === Opcode.NOP &&
          word1 === 0 &&
          word2 === 0 &&
          word3 === 0
        ) {
          return ['HALT', null, null, null];
        }
        // Special case for BRK (BRK with all zeros)
        if (
          opcode === Opcode.BRK &&
          word1 === 0 &&
          word2 === 0 &&
          word3 === 0
        ) {
          return ['BRK', null, null, null];
        }
        // R-type: rd, rs, rt
        return [
          mnemonic,
          this.getRegisterName(word1),
          this.getRegisterName(word2),
          this.getRegisterName(word3),
        ];

      case InstructionFormat.I:
        // I-type: rd, rs, imm
        return [
          mnemonic,
          this.getRegisterName(word1),
          this.getRegisterName(word2),
          word3.toString(),
        ];

      case InstructionFormat.I1:
        // I1-type: rd, imm
        return [mnemonic, this.getRegisterName(word1), word2.toString(), null];

      case InstructionFormat.I2:
        // I2-type: rd, imm1, imm2
        return [
          mnemonic,
          this.getRegisterName(word1),
          word2.toString(),
          word3.toString(),
        ];

      case InstructionFormat.J:
        // J-type: addr
        return [mnemonic, word1.toString(), null, null];

      default:
        return ['UNKNOWN', `0x${word0.toString(16)}`, null, null];
    }
  }

  disassembleToString(
    word0: number,
    word1: number,
    word2: number,
    word3: number,
  ): string {
    const [mnemonic, op1, op2, op3] = this.disassemble(
      word0,
      word1,
      word2,
      word3,
    );

    const parts = [mnemonic];
    if (op1 !== null) parts.push(op1);
    if (op2 !== null) parts.push(op2);
    if (op3 !== null) parts.push(op3);

    return parts.join(' ');
  }
}

export const disassembler = new Disassembler();
