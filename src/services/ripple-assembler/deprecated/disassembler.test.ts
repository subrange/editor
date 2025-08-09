import { describe, it, expect } from 'vitest';
import { Disassembler } from './disassembler';
import { InstructionEncoder } from './encoder';
import { Opcode, Register } from './types';

describe('Disassembler', () => {
  const disassembler = new Disassembler();
  const encoder = new InstructionEncoder();

  describe('R-type instructions', () => {
    it('should disassemble ADD instruction', () => {
      const inst = encoder.encodeR(Opcode.ADD, Register.R5, Register.R6, Register.R7);
      const result = disassembler.disassemble(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toEqual(['ADD', 'R5', 'R6', 'R7']);
    });

    it('should disassemble SUB instruction', () => {
      const inst = encoder.encodeR(Opcode.SUB, Register.R0, Register.PC, Register.RA);
      const result = disassembler.disassemble(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toEqual(['SUB', 'R0', 'PC', 'RA']);
    });

    it('should disassemble JALR instruction', () => {
      const inst = encoder.encodeR(Opcode.JALR, Register.RA, Register.R0, Register.R10);
      const result = disassembler.disassemble(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toEqual(['JALR', 'RA', 'R0', 'R10']);
    });

    it('should disassemble HALT (NOP with all zeros)', () => {
      const inst = encoder.encodeR(Opcode.NOP, 0, 0, 0);
      const result = disassembler.disassemble(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toEqual(['HALT', null, null, null]);
    });

    it('should disassemble NOP with non-zero operands', () => {
      const inst = encoder.encodeR(Opcode.NOP, 1, 0, 0);
      const result = disassembler.disassemble(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toEqual(['NOP', 'PC', 'R0', 'R0']);
    });
  });

  describe('I-type instructions', () => {
    it('should disassemble ADDI instruction', () => {
      const inst = encoder.encodeI(Opcode.ADDI, Register.R8, Register.R9, 42);
      const result = disassembler.disassemble(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toEqual(['ADDI', 'R8', 'R9', '42']);
    });

    it('should disassemble LOAD instruction', () => {
      const inst = encoder.encodeI(Opcode.LOAD, Register.R12, Register.R13, 1024);
      const result = disassembler.disassemble(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toEqual(['LOAD', 'R12', 'R13', '1024']);
    });

    it('should disassemble branch instruction BEQ', () => {
      const inst = encoder.encodeI(Opcode.BEQ, Register.R3, Register.R4, 8);
      const result = disassembler.disassemble(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toEqual(['BEQ', 'R3', 'R4', '8']);
    });
  });

  describe('I1-type instructions', () => {
    it('should disassemble LI instruction', () => {
      const inst = encoder.encodeI1(Opcode.LI, Register.R11, 65535);
      const result = disassembler.disassemble(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toEqual(['LI', 'R11', '65535', null]);
    });

    it('should disassemble LI with zero immediate', () => {
      const inst = encoder.encodeI1(Opcode.LI, Register.PCB, 0);
      const result = disassembler.disassemble(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toEqual(['LI', 'PCB', '0', null]);
    });
  });

  describe('disassembleToString', () => {
    it('should format R-type instruction as string', () => {
      const inst = encoder.encodeR(Opcode.XOR, Register.R14, Register.R15, Register.R0);
      const result = disassembler.disassembleToString(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toBe('XOR R14 R15 R0');
    });

    it('should format I-type instruction as string', () => {
      const inst = encoder.encodeI(Opcode.ORI, Register.RAB, Register.R5, 256);
      const result = disassembler.disassembleToString(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toBe('ORI RAB R5 256');
    });

    it('should format I1-type instruction as string', () => {
      const inst = encoder.encodeI1(Opcode.LI, Register.R6, 12345);
      const result = disassembler.disassembleToString(inst.word0, inst.word1, inst.word2, inst.word3);
      expect(result).toBe('LI R6 12345');
    });

    it('should format HALT instruction as string', () => {
      const result = disassembler.disassembleToString(Opcode.NOP, 0, 0, 0);
      expect(result).toBe('HALT');
    });
  });

  describe('unknown opcodes', () => {
    it('should handle unknown opcode', () => {
      const result = disassembler.disassemble(0xFF, 1, 2, 3);
      expect(result).toEqual(['UNKNOWN', '0xff', null, null]);
    });

    it('should format unknown opcode as string', () => {
      const result = disassembler.disassembleToString(0xFF, 1, 2, 3);
      expect(result).toBe('UNKNOWN 0xff');
    });
  });

  describe('round-trip assembly/disassembly', () => {
    it('should correctly round-trip all R-type instructions', () => {
      const rtypeOpcodes = [
        Opcode.ADD, Opcode.SUB, Opcode.AND, Opcode.OR, Opcode.XOR,
        Opcode.SLL, Opcode.SRL, Opcode.SLT, Opcode.SLTU, Opcode.JALR
      ];

      for (const opcode of rtypeOpcodes) {
        const inst = encoder.encodeR(opcode, Register.R10, Register.R11, Register.R12);
        const [mnemonic, op1, op2, op3] = disassembler.disassemble(inst.word0, inst.word1, inst.word2, inst.word3);
        
        expect(mnemonic).toBe(opcode === Opcode.JALR ? 'JALR' : Opcode[opcode]);
        expect(op1).toBe('R10');
        expect(op2).toBe('R11');
        expect(op3).toBe('R12');
      }
    });

    it('should correctly round-trip all I-type instructions', () => {
      const itypeOpcodes = [
        Opcode.ADDI, Opcode.ANDI, Opcode.ORI, Opcode.XORI,
        Opcode.SLLI, Opcode.SRLI, Opcode.LOAD, Opcode.STORE,
        Opcode.JAL, Opcode.BEQ, Opcode.BNE, Opcode.BLT, Opcode.BGE
      ];

      for (const opcode of itypeOpcodes) {
        const inst = encoder.encodeI(opcode, Register.R5, Register.R6, 1000);
        const [mnemonic, op1, op2, op3] = disassembler.disassemble(inst.word0, inst.word1, inst.word2, inst.word3);
        
        expect(mnemonic).toBe(Opcode[opcode]);
        expect(op1).toBe('R5');
        expect(op2).toBe('R6');
        expect(op3).toBe('1000');
      }
    });
  });
});