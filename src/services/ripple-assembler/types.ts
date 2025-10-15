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
  ADDI = 0x0a,
  ANDI = 0x0b,
  ORI = 0x0c,
  XORI = 0x0d,
  LI = 0x0e,
  SLLI = 0x0f,
  SRLI = 0x10,
  LOAD = 0x11,
  STORE = 0x12,
  JAL = 0x13,
  JALR = 0x14,
  BEQ = 0x15,
  BNE = 0x16,
  BLT = 0x17,
  BGE = 0x18,
  BRK = 0x19,
  MUL = 0x1a,
  DIV = 0x1b,
  MOD = 0x1c,
  MULI = 0x1d,
  DIVI = 0x1e,
  MODI = 0x1f,
}

export enum Register {
  R0 = 0, // Zero register
  PC = 1, // Program Counter
  PCB = 2, // Program Counter Bank
  RA = 3, // Return Address
  RAB = 4, // Return Address Bank
  RV0 = 5, // Return Value 0
  RV1 = 6, // Return Value 1
  A0 = 7, // Argument 0
  A1 = 8, // Argument 1
  A2 = 9, // Argument 2
  A3 = 10, // Argument 3
  X0 = 11, // Reserved/Extended 0
  X1 = 12, // Reserved/Extended 1
  X2 = 13, // Reserved/Extended 2
  X3 = 14, // Reserved/Extended 3
  T0 = 15, // Temporary 0
  T1 = 16, // Temporary 1
  T2 = 17, // Temporary 2
  T3 = 18, // Temporary 3
  T4 = 19, // Temporary 4
  T5 = 20, // Temporary 5
  T6 = 21, // Temporary 6
  T7 = 22, // Temporary 7
  S0 = 23, // Saved 0
  S1 = 24, // Saved 1
  S2 = 25, // Saved 2
  S3 = 26, // Saved 3
  SC = 27, // Allocator Scratch
  SB = 28, // Stack Bank
  SP = 29, // Stack Pointer
  FP = 30, // Frame Pointer
  GP = 31, // Global Pointer
}

export enum InstructionFormat {
  R,
  I,
  I1,
  I2,
  J,
}

export interface AssemblerOptions {
  caseInsensitive?: boolean;
  startBank?: number;
  bankSize?: number;
  maxImmediate?: number;
  memoryOffset?: number; // Offset for data section to account for memory-mapped regions (default 2)
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
  Data = 'data',
}

export interface AssemblerState {
  currentBank: number;
  currentOffset: number;
  labels: Map<string, Label>;
  dataLabels: Map<string, number>; // Labels in data section point to data offset
  pendingReferences: Map<
    number,
    { label: string; type: 'branch' | 'absolute' | 'data' }
  >;
  instructions: Instruction[];
  memoryData: number[];
  errors: string[];
}

export const DEFAULT_BANK_SIZE = 16;
export const INSTRUCTION_SIZE = 4;
export const DEFAULT_MAX_IMMEDIATE = 65535;

export const opcodeInfo: Record<
  Opcode,
  { format: InstructionFormat; mnemonic: string }
> = {
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
  [Opcode.BRK]: { format: InstructionFormat.R, mnemonic: 'BRK' },
  [Opcode.MUL]: { format: InstructionFormat.R, mnemonic: 'MUL' },
  [Opcode.DIV]: { format: InstructionFormat.R, mnemonic: 'DIV' },
  [Opcode.MOD]: { format: InstructionFormat.R, mnemonic: 'MOD' },
  [Opcode.MULI]: { format: InstructionFormat.I, mnemonic: 'MULI' },
  [Opcode.DIVI]: { format: InstructionFormat.I, mnemonic: 'DIVI' },
  [Opcode.MODI]: { format: InstructionFormat.I, mnemonic: 'MODI' },
};
