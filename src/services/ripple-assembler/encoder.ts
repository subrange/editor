import { Instruction, Opcode, InstructionFormat, opcodeInfo, DEFAULT_MAX_IMMEDIATE } from './types';

export class InstructionEncoder {
  private maxImmediate: number;

  constructor(maxImmediate: number = DEFAULT_MAX_IMMEDIATE) {
    this.maxImmediate = maxImmediate;
  }
  encodeR(opcode: Opcode, rd: number, rs: number, rt: number): Instruction {
    return {
      opcode,
      word0: opcode,
      word1: rd,
      word2: rs,
      word3: rt
    };
  }

  encodeI(opcode: Opcode, rd: number, rs: number, imm: number): Instruction {
    if (imm > this.maxImmediate || imm < 0) {
      throw new Error(`Immediate value ${imm} out of range (0-${this.maxImmediate})`);
    }
    return {
      opcode,
      word0: opcode,
      word1: rd,
      word2: rs,
      word3: imm
    };
  }

  encodeI1(opcode: Opcode, rd: number, imm: number): Instruction {
    if (imm > this.maxImmediate || imm < 0) {
      throw new Error(`Immediate value ${imm} out of range (0-${this.maxImmediate})`);
    }
    return {
      opcode,
      word0: opcode,
      word1: rd,
      word2: imm,
      word3: 0
    };
  }

  encodeI2(opcode: Opcode, rd: number, imm1: number, imm2: number): Instruction {
    if (imm1 > this.maxImmediate || imm1 < 0) {
      throw new Error(`Immediate value ${imm1} out of range (0-${this.maxImmediate})`);
    }
    if (imm2 > this.maxImmediate || imm2 < 0) {
      throw new Error(`Immediate value ${imm2} out of range (0-${this.maxImmediate})`);
    }
    return {
      opcode,
      word0: opcode,
      word1: rd,
      word2: imm1,
      word3: imm2
    };
  }

  encodeJ(opcode: Opcode, addr: number): Instruction {
    if (addr > 15 || addr < 0) {
      throw new Error(`Jump address ${addr} out of bank-local range (0-15)`);
    }
    return {
      opcode,
      word0: opcode,
      word1: addr,
      word2: 0,
      word3: 0
    };
  }

  encode(opcode: Opcode, operands: number[]): Instruction {
    const info = opcodeInfo[opcode];
    if (!info) {
      throw new Error(`Unknown opcode: ${opcode}`);
    }

    switch (info.format) {
      case InstructionFormat.R:
        if (operands.length !== 3) {
          throw new Error(`${info.mnemonic} requires 3 operands, got ${operands.length}`);
        }
        return this.encodeR(opcode, operands[0], operands[1], operands[2]);

      case InstructionFormat.I:
        if (operands.length !== 3) {
          throw new Error(`${info.mnemonic} requires 3 operands, got ${operands.length}`);
        }
        return this.encodeI(opcode, operands[0], operands[1], operands[2]);

      case InstructionFormat.I1:
        if (operands.length !== 2) {
          throw new Error(`${info.mnemonic} requires 2 operands, got ${operands.length}`);
        }
        return this.encodeI1(opcode, operands[0], operands[1]);

      case InstructionFormat.I2:
        if (operands.length !== 3) {
          throw new Error(`${info.mnemonic} requires 3 operands, got ${operands.length}`);
        }
        return this.encodeI2(opcode, operands[0], operands[1], operands[2]);

      case InstructionFormat.J:
        if (operands.length !== 1) {
          throw new Error(`${info.mnemonic} requires 1 operand, got ${operands.length}`);
        }
        return this.encodeJ(opcode, operands[0]);

      default:
        throw new Error(`Unsupported instruction format: ${info.format}`);
    }
  }

  encodeSpecial(mnemonic: string, operands: number[]): Instruction | null {
    const upperMnemonic = mnemonic.toUpperCase();
    
    if (upperMnemonic === 'HALT') {
      return this.encodeR(Opcode.NOP, 0, 0, 0);
    }

    if (upperMnemonic === 'JALR' && operands.length === 2) {
      return this.encodeR(Opcode.JALR, operands[0], 0, operands[1]);
    }

    return null;
  }
}