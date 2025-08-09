export interface Instruction {
  opcode: number;
  word0: number;
  word1: number;
  word2: number;
  word3: number;
}

export enum Opcode {
  NOP = 0x00,
  ADD = 0x01,
  SUB = 0x02,
  AND = 0x03,
  OR = 0x04,
  XOR = 0x05,
  SLL = 0x06,
  SRL = 0x07,
  SLT = 0x08,
  SLTU = 0x09,
  ADDI = 0x0A,
  ANDI = 0x0B,
  ORI = 0x0C,
  XORI = 0x0D,
  LI = 0x0E,
  SLLI = 0x0F,
  SRLI = 0x10,
  LOAD = 0x11,
  STORE = 0x12,
  JAL = 0x13,
  JALR = 0x14,
  BEQ = 0x15,
  BNE = 0x16,
  BLT = 0x17,
  BGE = 0x18,
  BRK = 0x19
}

export enum Register {
  R0 = 0,
  PC = 1,
  PCB = 2,
  RA = 3,
  RAB = 4,
  R3 = 5,
  R4 = 6,
  R5 = 7,
  R6 = 8,
  R7 = 9,
  R8 = 10,
  R9 = 11,
  R10 = 12,
  R11 = 13,
  R12 = 14,
  R13 = 15,
  R14 = 16,
  R15 = 17
}

export enum InstructionFormat {
  R,
  I,
  I1,
  I2,
  J
}

export interface AssemblerOptions {
  caseInsensitive?: boolean;
  startBank?: number;
  bankSize?: number;
  maxImmediate?: number;
  memoryOffset?: number;  // Offset for data section to account for memory-mapped regions (default 2)
}

export interface Label {
  name: string;
  bank: number;
  offset: number;
  absoluteAddress: number;
}

export interface AssemblerResult {
  instructions: Instruction[];
  labels: Map<string, Label>;
  dataLabels: Map<string, number>;
  memoryData: number[];
  errors: string[];
}

export interface ParsedLine {
  label?: string;
  mnemonic?: string;
  operands: string[];
  directive?: string;
  directiveArgs?: string[];
  lineNumber: number;
  raw: string;
}

export enum Section {
  Code = 'code',
  Data = 'data'
}

export interface AssemblerState {
  currentBank: number;
  currentOffset: number;
  labels: Map<string, Label>;
  dataLabels: Map<string, number>; // Labels in data section point to data offset
  pendingReferences: Map<number, { label: string; type: 'branch' | 'absolute' | 'data' }>;
  instructions: Instruction[];
  memoryData: number[];
  errors: string[];
}

export const DEFAULT_BANK_SIZE = 16;
export const INSTRUCTION_SIZE = 4;
export const DEFAULT_MAX_IMMEDIATE = 65535;

export const opcodeInfo: Record<Opcode, { format: InstructionFormat; mnemonic: string }> = {
  [Opcode.NOP]: { format: InstructionFormat.R, mnemonic: 'NOP' },
  [Opcode.ADD]: { format: InstructionFormat.R, mnemonic: 'ADD' },
  [Opcode.SUB]: { format: InstructionFormat.R, mnemonic: 'SUB' },
  [Opcode.AND]: { format: InstructionFormat.R, mnemonic: 'AND' },
  [Opcode.OR]: { format: InstructionFormat.R, mnemonic: 'OR' },
  [Opcode.XOR]: { format: InstructionFormat.R, mnemonic: 'XOR' },
  [Opcode.SLL]: { format: InstructionFormat.R, mnemonic: 'SLL' },
  [Opcode.SRL]: { format: InstructionFormat.R, mnemonic: 'SRL' },
  [Opcode.SLT]: { format: InstructionFormat.R, mnemonic: 'SLT' },
  [Opcode.SLTU]: { format: InstructionFormat.R, mnemonic: 'SLTU' },
  [Opcode.ADDI]: { format: InstructionFormat.I, mnemonic: 'ADDI' },
  [Opcode.ANDI]: { format: InstructionFormat.I, mnemonic: 'ANDI' },
  [Opcode.ORI]: { format: InstructionFormat.I, mnemonic: 'ORI' },
  [Opcode.XORI]: { format: InstructionFormat.I, mnemonic: 'XORI' },
  [Opcode.LI]: { format: InstructionFormat.I1, mnemonic: 'LI' },
  [Opcode.SLLI]: { format: InstructionFormat.I, mnemonic: 'SLLI' },
  [Opcode.SRLI]: { format: InstructionFormat.I, mnemonic: 'SRLI' },
  [Opcode.LOAD]: { format: InstructionFormat.I, mnemonic: 'LOAD' },
  [Opcode.STORE]: { format: InstructionFormat.I, mnemonic: 'STORE' },
  [Opcode.JAL]: { format: InstructionFormat.I, mnemonic: 'JAL' },
  [Opcode.JALR]: { format: InstructionFormat.R, mnemonic: 'JALR' },
  [Opcode.BEQ]: { format: InstructionFormat.I, mnemonic: 'BEQ' },
  [Opcode.BNE]: { format: InstructionFormat.I, mnemonic: 'BNE' },
  [Opcode.BLT]: { format: InstructionFormat.I, mnemonic: 'BLT' },
  [Opcode.BGE]: { format: InstructionFormat.I, mnemonic: 'BGE' },
  [Opcode.BRK]: { format: InstructionFormat.R, mnemonic: 'BRK' }
};