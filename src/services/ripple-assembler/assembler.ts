import type {
  AssemblerOptions,
  AssemblerResult,
  AssemblerState, Instruction,
  ParsedLine
} from './types.ts';
import {
  Opcode,
  Register,
  DEFAULT_BANK_SIZE,
  DEFAULT_MAX_IMMEDIATE,
  INSTRUCTION_SIZE,
  opcodeInfo,
  Section
} from './types.ts';
import { InstructionEncoder } from './encoder.ts';
import { Parser } from './parser.ts';
import { MacroFormatter } from './macro-formatter.ts';

export class RippleAssembler {
  private encoder: InstructionEncoder;
  private parser: Parser;
  private formatter: MacroFormatter;
  private options: Required<AssemblerOptions>;

  constructor(options: AssemblerOptions = {}) {
    this.options = {
      caseInsensitive: options.caseInsensitive ?? true,
      startBank: options.startBank ?? 0,
      bankSize: options.bankSize ?? DEFAULT_BANK_SIZE,
      maxImmediate: options.maxImmediate ?? DEFAULT_MAX_IMMEDIATE
    };
    this.encoder = new InstructionEncoder(this.options.maxImmediate);
    this.parser = new Parser(this.options.caseInsensitive);
    this.formatter = new MacroFormatter();
  }

  assemble(source: string): AssemblerResult {
    const state: AssemblerState = {
      currentBank: this.options.startBank,
      currentOffset: 0,
      labels: new Map(),
      dataLabels: new Map(),
      pendingReferences: new Map(),
      instructions: [],
      memoryData: [],
      errors: []
    };

    const allLines = this.parser.parseSource(source);
    
    // Process sections and directives
    const codeLines = this.processSections(allLines, state);
    
    if (state.errors.length === 0) {
      this.firstPass(codeLines, state);
    }
    
    if (state.errors.length === 0) {
      this.secondPass(codeLines, state);
    }
    
    if (state.errors.length === 0) {
      this.resolveReferences(state);
    }

    return {
      instructions: state.instructions,
      labels: state.labels,
      dataLabels: state.dataLabels,
      memoryData: state.memoryData,
      errors: state.errors
    };
  }

  private processSections(lines: ParsedLine[], state: AssemblerState): ParsedLine[] {
    let currentSection: Section = Section.Code;
    const codeLines: ParsedLine[] = [];
    let currentDataOffset = 0;
    
    for (const line of lines) {
      if (line.directive) {
        if (line.directive === 'data') {
          currentSection = Section.Data;
          continue;
        } else if (line.directive === 'code') {
          currentSection = Section.Code;
          continue;
        } else if (currentSection === Section.Data && !line.label) {
          // Handle standalone data directives
          const bytesAdded = this.processDataDirective(line, state);
          currentDataOffset += bytesAdded;
          continue;
        }
      }
      
      // Handle labels in data section
      if (currentSection === Section.Data && line.label) {
        state.dataLabels.set(line.label, currentDataOffset);
        // Process any directive on the same line as the label
        if (line.directive) {
          const bytesAdded = this.processDataDirective(line, state);
          currentDataOffset += bytesAdded;
        }
        continue;
      }
      
      // Regular lines go to code section
      if (currentSection === Section.Code) {
        codeLines.push(line);
      }
    }
    
    // Return code-only lines for further processing
    return codeLines;
  }

  private processDataDirective(line: ParsedLine, state: AssemblerState): number {
    if (!line.directive || !line.directiveArgs) return 0;
    
    let bytesAdded = 0;
    
    switch (line.directive) {
      case 'byte':
      case 'db':
        // .byte values or .db values
        for (const arg of line.directiveArgs) {
          const value = this.parseDataValue(arg, 1, line.lineNumber, state);
          if (value !== null) {
            state.memoryData.push(value);
            bytesAdded++;
          }
        }
        break;
        
      case 'word':
      case 'dw':
        // .word values or .dw values
        for (const arg of line.directiveArgs) {
          const value = this.parseDataValue(arg, 2, line.lineNumber, state);
          if (value !== null) {
            state.memoryData.push(value);
            bytesAdded++; // Each word counts as 1 cell in the data array
          }
        }
        break;
        
      case 'string':
      case 'ascii':
        // .string "text" or .ascii "text"
        if (line.directiveArgs.length > 0) {
          const stringValue = this.parseString(line.directiveArgs.join(' '));
          for (const char of stringValue) {
            state.memoryData.push(char.charCodeAt(0));
            bytesAdded++;
          }
        }
        break;
        
      case 'asciiz':
        // .asciiz "text" - null terminated string
        if (line.directiveArgs.length > 0) {
          const stringValue = this.parseString(line.directiveArgs.join(' '));
          for (const char of stringValue) {
            state.memoryData.push(char.charCodeAt(0));
            bytesAdded++;
          }
          state.memoryData.push(0); // null terminator
          bytesAdded++;
        }
        break;
        
      case 'space':
      case 'zero':
        // .space n or .zero n - reserve n bytes of zeros
        if (line.directiveArgs.length > 0) {
          const count = this.parseDataValue(line.directiveArgs[0], 2, line.lineNumber, state);
          if (count !== null) {
            for (let i = 0; i < count; i++) {
              state.memoryData.push(0);
              bytesAdded++;
            }
          }
        }
        break;
        
      default:
        state.errors.push(`Line ${line.lineNumber}: Unknown directive: .${line.directive}`);
    }
    
    return bytesAdded;
  }
  
  
  private parseDataValue(value: string, size: number, lineNumber: number, state: AssemblerState): number | null {
    // Remove quotes if present
    if ((value.startsWith('"') && value.endsWith('"')) || 
        (value.startsWith("'") && value.endsWith("'"))) {
      // Character literal
      const char = value.substring(1, value.length - 1);
      if (char.length === 1) {
        return char.charCodeAt(0);
      } else if (char.length === 2 && char[0] === '\\') {
        // Escape sequences
        switch (char[1]) {
          case 'n': return 10;
          case 'r': return 13;
          case 't': return 9;
          case '0': return 0;
          case '\\': return 92;
          default:
            state.errors.push(`Line ${lineNumber}: Unknown escape sequence: ${char}`);
            return null;
        }
      }
    }
    
    // Try to parse as number
    let num: number;
    if (value.startsWith('0x')) {
      num = parseInt(value, 16);
    } else if (value.startsWith('0b')) {
      num = parseInt(value.substring(2), 2);
    } else {
      num = parseInt(value, 10);
    }
    
    if (isNaN(num)) {
      state.errors.push(`Line ${lineNumber}: Invalid data value: ${value}`);
      return null;
    }
    
    const maxValue = size === 1 ? 255 : this.options.maxImmediate;
    if (num < 0 || num > maxValue) {
      state.errors.push(`Line ${lineNumber}: Data value out of range: ${value}`);
      return null;
    }
    
    return num;
  }
  
  private parseString(value: string): string {
    // Remove surrounding quotes
    if ((value.startsWith('"') && value.endsWith('"')) || 
        (value.startsWith("'") && value.endsWith("'"))) {
      value = value.substring(1, value.length - 1);
    }
    
    // Process escape sequences
    return value.replace(/\\(.)/g, (match, char) => {
      switch (char) {
        case 'n': return '\n';
        case 'r': return '\r';
        case 't': return '\t';
        case '0': return '\0';
        case '\\': return '\\';
        case '"': return '"';
        case "'": return "'";
        default: return char;
      }
    });
  }

  private firstPass(lines: ParsedLine[], state: AssemblerState): void {
    const cellsPerBank = this.options.bankSize * INSTRUCTION_SIZE;
    
    for (const line of lines) {
      if (line.label) {
        const absoluteAddress = state.currentBank * cellsPerBank + state.currentOffset * INSTRUCTION_SIZE;
        
        if (state.labels.has(line.label)) {
          state.errors.push(`Line ${line.lineNumber}: Duplicate label '${line.label}'`);
          continue;
        }
        
        state.labels.set(line.label, {
          name: line.label,
          bank: state.currentBank,
          offset: state.currentOffset,
          absoluteAddress
        });
      }

      if (line.mnemonic) {
        state.currentOffset++;
        
        if (state.currentOffset >= this.options.bankSize) {
          state.currentOffset = 0;
          state.currentBank++;
        }
      }
    }
  }

  private secondPass(lines: ParsedLine[], state: AssemblerState): void {
    state.currentBank = this.options.startBank;
    state.currentOffset = 0;
    state.instructions = [];

    for (const line of lines) {
      if (!line.mnemonic) {
        continue;
      }

      try {
        const instruction = this.assembleInstruction(line, state);
        state.instructions.push(instruction);
        
        state.currentOffset++;
        if (state.currentOffset >= this.options.bankSize) {
          state.currentOffset = 0;
          state.currentBank++;
        }
      } catch (error) {
        state.errors.push(`Line ${line.lineNumber}: ${error.message}`);
      }
    }
  }

  private assembleInstruction(line: ParsedLine, state: AssemblerState): any {
    const mnemonic = line.mnemonic!.toUpperCase();
    
    // Handle special cases first
    if (mnemonic === 'NOP' || mnemonic === 'HALT') {
      return this.encoder.encodeR(Opcode.NOP, 0, 0, 0);
    }

    if (mnemonic === 'JALR' && line.operands.length === 2) {
      const operands = this.parseOperands(line.operands, Opcode.JALR, state);
      return this.encoder.encodeSpecial(mnemonic, operands as number[]);
    }

    // Handle LOAD special case - LOAD rd, rs, offset means load from [rs+offset]
    // But the VM expects LOAD rd, bank, addr format
    if (mnemonic === 'LOAD' && line.operands.length === 3) {
      const rd = this.parseRegister(line.operands[0]);
      const rs = this.parseRegister(line.operands[1]); 
      const offset = this.parseImmediate(line.operands[2]);
      
      // For now, ignore offset and use rs as the address
      // LOAD R3, R5, 0 -> word1=R3, word2=0 (bank), word3=R5 (addr register)
      return {
        opcode: Opcode.LOAD,
        word0: Opcode.LOAD,
        word1: rd,
        word2: 0, // bank (always 0 for now)
        word3: rs // address register
      };
    }

    const opcode = this.getOpcodeFromMnemonic(mnemonic);
    
    const operands = this.parseOperands(line.operands, opcode, state);

    const isBranch = [Opcode.BEQ, Opcode.BNE, Opcode.BLT, Opcode.BGE].includes(opcode);

    if (isBranch) {
      const labelOperandIndex = 2;
      const labelName = line.operands[labelOperandIndex];
      
      if (this.isLabel(labelName)) {
        const instructionIndex = state.instructions.length;
        state.pendingReferences.set(instructionIndex, {
          label: labelName,
          type: 'branch'
        });
      }
    }

    return this.encoder.encode(opcode, operands);
  }

  private parseOperands(operands: string[], opcode: Opcode, state: AssemblerState): number[] {
    const parsed: number[] = [];
    const isBranch = [Opcode.BEQ, Opcode.BNE, Opcode.BLT, Opcode.BGE].includes(opcode);

    for (let i = 0; i < operands.length; i++) {
      const operand = operands[i];
      const isLastOperand = i === operands.length - 1;
      const isBranchTarget = isBranch && isLastOperand;
      const isJumpTarget = opcode === Opcode.JAL && i === 2;

      if (this.isRegister(operand)) {
        parsed.push(this.parseRegister(operand));
      } else if (this.isLabel(operand)) {
        if (isBranchTarget || isJumpTarget) {
          // Code label - will be resolved later
          parsed.push(0);
          // Mark for resolution
          const instructionIndex = state.instructions.length;
          state.pendingReferences.set(instructionIndex, {
            label: operand,
            type: isBranchTarget ? 'branch' : 'jump',
            operandIndex: i
          });
        } else {
          // Could be a data label - check if it exists
          if (state.dataLabels.has(operand)) {
            parsed.push(state.dataLabels.get(operand)!);
          } else {
            // Assume it's a forward reference to a code label or undefined
            parsed.push(0);
            // Mark for resolution
            const instructionIndex = state.instructions.length;
            state.pendingReferences.set(instructionIndex, {
              label: operand,
              type: 'data'
            });
          }
        }
      } else if (this.isImmediate(operand)) {
        parsed.push(this.parseImmediate(operand));
      } else {
        throw new Error(`Invalid operand: ${operand}`);
      }
    }

    return parsed;
  }

  private resolveReferences(state: AssemblerState): void {
    for (const [index, ref] of state.pendingReferences) {
      if (ref.type === 'data') {
        // Try to resolve data label
        const dataOffset = state.dataLabels.get(ref.label);
        if (dataOffset !== undefined) {
          // For now, we'll just use the data offset directly
          // In a real system, this would need to account for where data is loaded in memory
          const instruction = state.instructions[index];
          // Find which word contains the placeholder
          if (instruction.word1 === 0) instruction.word1 = dataOffset;
          else if (instruction.word2 === 0) instruction.word2 = dataOffset;
          else if (instruction.word3 === 0) instruction.word3 = dataOffset;
          continue;
        }
      }
      
      const label = state.labels.get(ref.label);
      if (!label) {
        state.errors.push(`Unresolved label: ${ref.label}`);
        continue;
      }

      const instruction = state.instructions[index];
      
      if (ref.type === 'branch') {
        const currentBank = Math.floor(index / this.options.bankSize);
        const currentOffset = index % this.options.bankSize;
        
        if (label.bank !== currentBank) {
          state.errors.push(`Branch to label '${ref.label}' crosses bank boundary`);
          continue;
        }
        
        const relativeOffset = label.offset - currentOffset - 1;
        // For 16-bit signed immediate, the range should be -32768 to 32767
        if (relativeOffset < -32768 || relativeOffset > 32767) {
          state.errors.push(`Branch offset to '${ref.label}' out of range: ${relativeOffset}`);
          continue;
        }
        
        instruction.word3 = relativeOffset & 0xFFFF;
      } else if (ref.type === 'jump') {
        // JAL instruction - absolute address goes in word3
        if (label.bank !== Math.floor(index / this.options.bankSize)) {
          const farJumpIndex = index;
          const loadPCBInstruction = this.encoder.encodeI1(Opcode.LI, Register.PCB, label.bank);
          const jalInstruction = this.encoder.encodeI2(Opcode.JAL, label.offset, 0, 0);
          jalInstruction.word1 = label.offset;
          
          state.instructions.splice(farJumpIndex, 1, loadPCBInstruction, jalInstruction);
          state.errors.push(`Far jump to '${ref.label}' requires manual bank management`);
        } else {
          // For JAL, the address goes in word3 (third operand)
          instruction.word3 = label.offset;
        }
      } else {
        // Default handling for other types
        instruction.word1 = label.offset;
      }
    }
  }

  private getOpcodeFromMnemonic(mnemonic: string): Opcode {
    const upper = mnemonic.toUpperCase();
    
    for (const [opcodeValue, info] of Object.entries(opcodeInfo)) {
      if (info.mnemonic === upper) {
        return Number(opcodeValue) as Opcode;
      }
    }
    
    if (upper === 'HALT') {
      return Opcode.NOP;
    }
    
    throw new Error(`Unknown mnemonic: ${mnemonic}`);
  }

  private isRegister(operand: string): boolean {
    const upper = operand.toUpperCase();
    // Check if it matches R\d+ pattern or is a named register like PC, RA, etc.
    // Important: We need to check the keys of Register enum, not values
    const registerNames = Object.keys(Register).filter(k => isNaN(Number(k)));
    return /^R\d+$/.test(upper) || registerNames.includes(upper);
  }

  private isLabel(operand: string): boolean {
    return /^[A-Za-z_][A-Za-z0-9_]*$/.test(operand);
  }

  private isImmediate(operand: string): boolean {
    return /^-?\d+$/.test(operand) || /^0x[0-9A-Fa-f]+$/.test(operand);
  }

  private parseRegister(operand: string): number {
    const upper = operand.toUpperCase();
    
    if (upper in Register) {
      return Register[upper as keyof typeof Register];
    }
    
    const match = upper.match(/^R(\d+)$/);
    if (match) {
      const num = parseInt(match[1], 10);
      if (num >= 0 && num <= 17) {
        return num;
      }
    }
    
    throw new Error(`Invalid register: ${operand}`);
  }

  private parseImmediate(operand: string): number {
    let value: number;
    
    if (operand.startsWith('0x')) {
      value = parseInt(operand, 16);
    } else {
      value = parseInt(operand, 10);
    }
    
    if (value < 0) {
      value = (value + 65536) % 65536;
    }
    
    if (value > this.options.maxImmediate) {
      throw new Error(`Immediate value out of range: ${operand}`);
    }
    
    return value;
  }

  toMachineCode(instructions: Instruction[]): number[] {
    const code: number[] = [];
    
    for (const inst of instructions) {
      code.push(inst.word0, inst.word1, inst.word2, inst.word3);
    }
    
    return code;
  }

  toBinary(instructions: Instruction[]): Uint16Array {
    const code = this.toMachineCode(instructions);
    return new Uint16Array(code);
  }

  toMacroFormat(instructions: Instruction[], comments?: Map<number, string>): string {
    return this.formatter.formatProgram(instructions, comments);
  }

  toFullMacroFormat(
    instructions: Instruction[], 
    data?: number[], 
    comments?: Map<number, string>,
    header?: string
  ): string {
    return this.formatter.formatFullProgram(instructions, data, comments, header);
  }

  getBankSize(): number {
    return this.options.bankSize;
  }

  getMaxImmediate(): number {
    return this.options.maxImmediate;
  }

  getCellsPerBank(): number {
    return this.options.bankSize * INSTRUCTION_SIZE;
  }
}