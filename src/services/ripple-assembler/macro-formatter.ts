import { Instruction, Opcode, Register } from './types';

export class MacroFormatter {
  private opcodeToMacro: Map<number, string> = new Map([
    [Opcode.NOP, '@OP_NOP'],
    [Opcode.ADD, '@OP_ADD'],
    [Opcode.SUB, '@OP_SUB'],
    [Opcode.AND, '@OP_AND'],
    [Opcode.OR, '@OP_OR'],
    [Opcode.XOR, '@OP_XOR'],
    [Opcode.SLL, '@OP_SLL'],
    [Opcode.SRL, '@OP_SRL'],
    [Opcode.SLT, '@OP_SLT'],
    [Opcode.SLTU, '@OP_SLTU'],
    [Opcode.ADDI, '@OP_ADDI'],
    [Opcode.ANDI, '@OP_ANDI'],
    [Opcode.ORI, '@OP_ORI'],
    [Opcode.XORI, '@OP_XORI'],
    [Opcode.LI, '@OP_LI'],
    [Opcode.SLLI, '@OP_SLLI'],
    [Opcode.SRLI, '@OP_SRLI'],
    [Opcode.LOAD, '@OP_LOAD'],
    [Opcode.STORE, '@OP_STOR'],
    [Opcode.JAL, '@OP_JAL'],
    [Opcode.JALR, '@OP_JALR'],
    [Opcode.BEQ, '@OP_BEQ'],
    [Opcode.BNE, '@OP_BNE'],
    [Opcode.BLT, '@OP_BLT'],
    [Opcode.BGE, '@OP_BGE'],
  ]);

  private registerToMacro(reg: number): string {
    switch (reg) {
      case Register.R0: return '@R0';
      case Register.PC: return '@PC';
      case Register.PCB: return '@PCB';
      case Register.RA: return '@RA';
      case Register.RAB: return '@RAB';
      default:
        if (reg >= 5 && reg <= 17) {
          return `@R${reg - 2}`; // R3-R15
        }
        return `#R${reg}`;
    }
  }

  formatInstruction(instruction: Instruction, isFirst: boolean = false): string {
    let opcodeMacro = this.opcodeToMacro.get(instruction.opcode) || '@OP_NOP';
    
    // Special case for HALT (encoded as NOP 0,0,0)
    if (instruction.opcode === Opcode.NOP && 
        instruction.word1 === 0 && 
        instruction.word2 === 0 && 
        instruction.word3 === 0) {
      opcodeMacro = '@OP_HALT';
    }

    const formatOperand = (value: number, isRegister: boolean = false): string => {
      if (isRegister) {
        return this.registerToMacro(value);
      }
      return value.toString();
    };

    // Determine which operands are registers based on opcode
    const isRFormat = [
      Opcode.ADD, Opcode.SUB, Opcode.AND, Opcode.OR, Opcode.XOR,
      Opcode.SLL, Opcode.SRL, Opcode.SLT, Opcode.SLTU, Opcode.JALR
    ].includes(instruction.opcode);

    const isIFormat = [
      Opcode.ADDI, Opcode.ANDI, Opcode.ORI, Opcode.XORI,
      Opcode.SLLI, Opcode.SRLI, Opcode.LOAD, Opcode.STORE,
      Opcode.BEQ, Opcode.BNE, Opcode.BLT, Opcode.BGE
    ].includes(instruction.opcode);


    let word1Str: string;
    let word2Str: string;
    let word3Str: string;

    if (isRFormat) {
      word1Str = formatOperand(instruction.word1, true);
      word2Str = formatOperand(instruction.word2, true);
      word3Str = formatOperand(instruction.word3, true);
    } else if (isIFormat) {
      // Special case for LOAD: all positions can be registers
      if (instruction.opcode === Opcode.LOAD) {
        word1Str = formatOperand(instruction.word1, true);   // rd is register
        word2Str = formatOperand(instruction.word2, true);   // bank is register (or 0)
        word3Str = formatOperand(instruction.word3, true);   // addr is register
      } else {
        word1Str = formatOperand(instruction.word1, true);
        word2Str = formatOperand(instruction.word2, true);
        word3Str = formatOperand(instruction.word3, false);
      }
    } else if (instruction.opcode === Opcode.LI) {
      word1Str = formatOperand(instruction.word1, true);
      word2Str = formatOperand(instruction.word2, false);
      word3Str = formatOperand(instruction.word3, false);
    } else if (instruction.opcode === Opcode.JAL) {
      // JAL uses address in word1
      word1Str = formatOperand(instruction.word1, false);
      word2Str = formatOperand(instruction.word2, false);
      word3Str = formatOperand(instruction.word3, false);
    } else {
      word1Str = formatOperand(instruction.word1, false);
      word2Str = formatOperand(instruction.word2, false);
      word3Str = formatOperand(instruction.word3, false);
    }

    const cmdType = isFirst ? '@program_start' : '@cmd';
    
    return `${cmdType}(${opcodeMacro.padEnd(10)}, ${word1Str.padEnd(4)}, ${word2Str.padEnd(4)}, ${word3Str})`;
  }

  formatProgram(instructions: Instruction[], comments?: Map<number, string>): string {
    const lines: string[] = [];
    
    instructions.forEach((instruction, index) => {
      const comment = comments?.get(index);
      const formattedInst = this.formatInstruction(instruction, index === 0);
      
      if (comment) {
        lines.push(`${formattedInst}        // ${comment}`);
      } else {
        lines.push(formattedInst);
      }
    });
    
    lines.push('@program_end');
    
    return lines.join('\n');
  }

  formatDataSection(data: number[]): string {
    const lines: string[] = [];
    
    lines.push(`// Data segment`);
    lines.push(`@lane(#L_MEM,`);
    
    for (const value of data) {
      lines.push(`  @set(${value}) @nextword`);
    }
    
    lines.push(`)`);
    
    return lines.join('\n');
  }

  formatFullProgram(
    instructions: Instruction[], 
    data?: number[], 
    comments?: Map<number, string>,
    header?: string
  ): string {
    const sections: string[] = [];
    
    if (header) {
      sections.push(`// ${header}`);
      sections.push('');
    }
    
    if (data && data.length > 0) {
      sections.push(this.formatDataSection(data));
      sections.push('');
    }
    
    sections.push('// Program');
    sections.push(this.formatProgram(instructions, comments));
    
    return sections.join('\n');
  }
}