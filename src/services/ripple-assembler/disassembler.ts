import { Opcode, InstructionFormat, opcodeInfo, Register } from './types.ts';

export type DisassembledInstruction = [string, string | null, string | null, string | null];

const registerNames: Record<number, string> = {
  [Register.R0]: 'R0',
  [Register.PC]: 'PC',
  [Register.PCB]: 'PCB',
  [Register.RA]: 'RA',
  [Register.RAB]: 'RAB',
  [Register.R3]: 'R3',
  [Register.R4]: 'R4',
  [Register.R5]: 'R5',
  [Register.R6]: 'R6',
  [Register.R7]: 'R7',
  [Register.R8]: 'R8',
  [Register.R9]: 'R9',
  [Register.R10]: 'R10',
  [Register.R11]: 'R11',
  [Register.R12]: 'R12',
  [Register.R13]: 'R13',
  [Register.R14]: 'R14',
  [Register.R15]: 'R15'
};

export class Disassembler {
  private getRegisterName(reg: number): string {
    return registerNames[reg] || `R${reg}`;
  }

  disassemble(word0: number, word1: number, word2: number, word3: number): DisassembledInstruction {
    const opcode = word0 as Opcode;
    const info = opcodeInfo[opcode];
    
    if (!info) {
      return ['UNKNOWN', `0x${word0.toString(16)}`, null, null];
    }

    const mnemonic = info.mnemonic;
    
    switch (info.format) {
      case InstructionFormat.R:
        // Special case for HALT (NOP with all zeros)
        if (opcode === Opcode.NOP && word1 === 0 && word2 === 0 && word3 === 0) {
          return ['HALT', null, null, null];
        }
        // R-type: rd, rs, rt
        return [
          mnemonic,
          this.getRegisterName(word1),
          this.getRegisterName(word2),
          this.getRegisterName(word3)
        ];

      case InstructionFormat.I:
        // I-type: rd, rs, imm
        return [
          mnemonic,
          this.getRegisterName(word1),
          this.getRegisterName(word2),
          word3.toString()
        ];

      case InstructionFormat.I1:
        // I1-type: rd, imm
        return [
          mnemonic,
          this.getRegisterName(word1),
          word2.toString(),
          null
        ];

      case InstructionFormat.I2:
        // I2-type: rd, imm1, imm2
        return [
          mnemonic,
          this.getRegisterName(word1),
          word2.toString(),
          word3.toString()
        ];

      case InstructionFormat.J:
        // J-type: addr
        return [
          mnemonic,
          word1.toString(),
          null,
          null
        ];

      default:
        return ['UNKNOWN', `0x${word0.toString(16)}`, null, null];
    }
  }

  disassembleToString(word0: number, word1: number, word2: number, word3: number): string {
    const [mnemonic, op1, op2, op3] = this.disassemble(word0, word1, word2, word3);
    
    const parts = [mnemonic];
    if (op1 !== null) parts.push(op1);
    if (op2 !== null) parts.push(op2);
    if (op3 !== null) parts.push(op3);
    
    return parts.join(' ');
  }
}

export const disassembler = new Disassembler();