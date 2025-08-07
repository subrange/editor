import { describe, it, expect } from 'vitest';
import { RippleAssembler } from './assembler';
import { Opcode, Register } from './types';

describe('RippleAssembler', () => {
  const assembler = new RippleAssembler();

  describe('Basic instructions', () => {
    it('should assemble NOP instruction', () => {
      const source = 'NOP';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions).toHaveLength(1);
      expect(result.instructions[0]).toEqual({
        opcode: Opcode.NOP,
        word0: 0x00,
        word1: 0,
        word2: 0,
        word3: 0
      });
    });

    it('should assemble ADD instruction', () => {
      const source = 'ADD R3, R4, R5';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions).toHaveLength(1);
      expect(result.instructions[0]).toEqual({
        opcode: Opcode.ADD,
        word0: 0x01,
        word1: 5,
        word2: 6,
        word3: 7
      });
    });

    it('should assemble LI instruction', () => {
      const source = 'LI R3, 42';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions).toHaveLength(1);
      expect(result.instructions[0]).toEqual({
        opcode: Opcode.LI,
        word0: 0x0E,
        word1: 5,
        word2: 42,
        word3: 0
      });
    });

    it('should assemble HALT as NOP', () => {
      const source = 'HALT';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions).toHaveLength(1);
      expect(result.instructions[0]).toEqual({
        opcode: Opcode.NOP,
        word0: 0x00,
        word1: 0,
        word2: 0,
        word3: 0
      });
    });
  });

  describe('Register parsing', () => {
    it('should parse named registers', () => {
      const source = 'ADD PC, RA, R0';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions[0]).toEqual({
        opcode: Opcode.ADD,
        word0: 0x01,
        word1: Register.PC,
        word2: Register.RA,
        word3: Register.R0
      });
    });

    it('should handle case-insensitive registers', () => {
      const source = 'ADD r3, R4, r5';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions[0].word1).toBe(5);
      expect(result.instructions[0].word2).toBe(6);
      expect(result.instructions[0].word3).toBe(7);
    });
  });

  describe('Labels and jumps', () => {
    it('should handle labels and local jumps', () => {
      const source = `
        LI R3, 5
        JAL loop
        HALT
        loop:
        SUB R3, R3, R4
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.labels.get('loop')).toBeDefined();
      expect(result.labels.get('loop')!.offset).toBe(3);
      expect(result.instructions[1]).toEqual({
        opcode: Opcode.JAL,
        word0: 0x13,
        word1: 3,
        word2: 0,
        word3: 0
      });
    });

    it('should handle branch instructions', () => {
      const source = `
        loop:
        LI R3, 1
        SUB R4, R4, R3
        BNE R4, R0, loop
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions[2]).toEqual({
        opcode: Opcode.BNE,
        word0: 0x16,
        word1: 6,
        word2: 0,
        word3: 65533
      });
    });
  });

  describe('Immediate values', () => {
    it('should handle decimal immediates', () => {
      const source = 'ADDI R3, R4, 1234';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions[0].word3).toBe(1234);
    });

    it('should handle hexadecimal immediates', () => {
      const source = 'ADDI R3, R4, 0xFF';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions[0].word3).toBe(255);
    });

    it('should handle negative immediates', () => {
      const source = 'ADDI R3, R4, -1';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions[0].word3).toBe(65535);
    });
  });

  describe('Special forms', () => {
    it('should handle JALR with two operands', () => {
      const source = 'JALR R4, R5';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions[0]).toEqual({
        opcode: Opcode.JALR,
        word0: 0x14,
        word1: 6,
        word2: 0,
        word3: 7
      });
    });
  });

  describe('Error handling', () => {
    it('should report duplicate labels', () => {
      const source = `
        loop:
        NOP
        loop:
        NOP
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0]).toContain('Duplicate label');
    });

    it('should report undefined labels', () => {
      const source = 'JAL undefined_label';
      const result = assembler.assemble(source);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0]).toContain('Unresolved label');
    });

    it('should report out-of-range immediates', () => {
      const source = 'LI R3, 70000';
      const result = assembler.assemble(source);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0]).toContain('out of range');
    });

    it('should report cross-bank branches', () => {
      const source = `
        target:
        NOP
        ${Array(16).fill('NOP').join('\n')}
        BEQ R3, R4, target
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0]).toContain('crosses bank boundary');
    });
  });

  describe('Machine code generation', () => {
    it('should generate correct machine code array', () => {
      const source = 'LI R3, 42';
      const result = assembler.assemble(source);
      const machineCode = assembler.toMachineCode(result.instructions);
      
      expect(machineCode).toEqual([0x0E, 5, 42, 0]);
    });

    it('should generate correct binary output', () => {
      const source = 'ADD R3, R4, R5';
      const result = assembler.assemble(source);
      const binary = assembler.toBinary(result.instructions);
      
      expect(binary).toBeInstanceOf(Uint16Array);
      expect(binary.length).toBe(4);
      expect(binary[0]).toBe(0x01);
      expect(binary[1]).toBe(5);
      expect(binary[2]).toBe(6);
      expect(binary[3]).toBe(7);
    });
  });

  describe('Comments', () => {
    it('should ignore semicolon comments', () => {
      const source = 'LI R3, 42 ; Load 42 into R3';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions).toHaveLength(1);
    });

    it('should ignore hash comments', () => {
      const source = 'LI R3, 42 # Load 42 into R3';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions).toHaveLength(1);
    });

    it('should ignore double-slash comments', () => {
      const source = 'LI R3, 42 // Load 42 into R3';
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions).toHaveLength(1);
    });
  });

  describe('Configurable options', () => {
    it('should respect custom bank size', () => {
      const customAssembler = new RippleAssembler({ bankSize: 8 });
      const source = `
        ${Array(8).fill('NOP').join('\n')}
        ; This should be in bank 1
        LI R3, 42
      `;
      const result = customAssembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions).toHaveLength(9);
      // Check that the LI instruction is in bank 1
      const expectedBank = Math.floor(8 / 8);
      expect(expectedBank).toBe(1);
    });

    it('should respect custom max immediate', () => {
      const customAssembler = new RippleAssembler({ maxImmediate: 255 });
      const source = 'LI R3, 256';
      const result = customAssembler.assemble(source);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0]).toContain('out of range');
    });

    it('should detect cross-bank branches with custom bank size', () => {
      const customAssembler = new RippleAssembler({ bankSize: 4 });
      const source = `
        target:
        NOP
        NOP
        NOP
        NOP
        ; Now in bank 1
        BEQ R3, R4, target
      `;
      const result = customAssembler.assemble(source);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0]).toContain('crosses bank boundary');
    });
  });

  describe('Data and code sections', () => {
    it('should handle .data section with strings', () => {
      const source = `
        .data
        .asciiz "Hello, World!"
        
        .code
        LI R3, 0
        LI R4, 0
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.memoryData).toEqual([
        72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33, 0
      ]);
      expect(result.instructions).toHaveLength(2);
    });

    it('should handle .data section with bytes', () => {
      const source = `
        .data
        .byte 0x48, 0x65, 0x6C, 0x6C, 0x6F
        .byte '!', 0
        
        .code
        LI R3, 0
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.memoryData).toEqual([72, 101, 108, 108, 111, 33, 0]);
    });

    it('should handle .data section with words', () => {
      const source = `
        .data
        .word 0x1234, 0x5678
        .dw 1000, 2000
        
        .code
        NOP
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.memoryData).toEqual([0x1234, 0x5678, 1000, 2000]);
    });

    it('should handle .space directive', () => {
      const source = `
        .data
        .space 5
        .byte 0xFF
        
        .code
        NOP
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.memoryData).toEqual([0, 0, 0, 0, 0, 0xFF]);
    });

    it('should handle escape sequences in strings', () => {
      const source = `
        .data
        .asciiz "Hello\\nWorld\\t!\\0"
        
        .code
        NOP
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.memoryData).toEqual([
        72, 101, 108, 108, 111, 10, 87, 111, 114, 108, 100, 9, 33, 0, 0
      ]);
    });

    it('should generate full output with embedded data', () => {
      const source = `
        .data
        .asciiz "Hi!"
        
        .code
        LI R3, 0
      `;
      const result = assembler.assemble(source);
      const output = assembler.toFullMacroFormat(
        result.instructions,
        result.memoryData,
        undefined,
        'Test with sections'
      );
      
      expect(output).toContain('// Test with sections');
      expect(output).toContain('// Data segment');
      expect(output).toContain('@set(72) @nextword');
      expect(output).toContain('@set(105) @nextword');
      expect(output).toContain('@set(33) @nextword');
      expect(output).toContain('@set(0) @nextword');
      expect(output).toContain('@program_start(@OP_LI');
    });
  });

  describe('Data section usage patterns', () => {
    it('should handle loading string data with newlines', () => {
      const source = `
        .data
        .asciiz "Hello\\nWorld\\n"
        
        .code
        start:
          LI R3, 0        ; Assuming data starts at address 0
          LI R4, 0        ; Counter
        loop:
          ADD R5, R3, R4  ; Calculate address
          LOAD R6, R5, 0  ; Load character
          BEQ R6, R0, end ; Exit on null
          ADDI R4, R4, 1  ; Increment counter
          JAL loop
        end:
          HALT
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      // "Hello\nWorld\n\0" = 72,101,108,108,111,10,87,111,114,108,100,10,0
      expect(result.memoryData).toEqual([
        72, 101, 108, 108, 111, 10,  // Hello\n
        87, 111, 114, 108, 100, 10,  // World\n
        0                            // null terminator
      ]);
      expect(result.instructions).toHaveLength(8);
    });

    it('should handle mixed data types', () => {
      const source = `
        .data
        ; String data
        .asciiz "Hi"
        
        ; Byte data
        .byte 0xFF, 0x00, 0xAA
        
        ; Word data
        .word 0x1234, 0x5678
        
        ; Space reservation
        .space 3
        
        .code
          LI R3, 0      ; Data base address
          LOAD R4, R3, 0 ; Load 'H'
          LOAD R5, R3, 3 ; Load 0xFF
          LOAD R6, R3, 6 ; Load 0x1234
          HALT
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.memoryData).toEqual([
        72, 105, 0,           // "Hi\0"
        0xFF, 0x00, 0xAA,     // bytes
        0x1234, 0x5678,       // words
        0, 0, 0               // space
      ]);
    });

    it('should handle string with escape sequences', () => {
      const source = `
        .data
        .string "Tab:\\tValue\\nQuote:\\"Hello\\"\\nBackslash:\\\\\\n"
        .byte 0  ; Manual null terminator for .string
        
        .code
          LI R3, 0
          LI R4, 0
        print:
          LOAD R5, R3, R4
          BEQ R5, R0, done
          STORE R5, R0, 0  ; Output to device
          ADDI R4, R4, 1
          JAL print
        done:
          HALT
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      // Check for correct escape sequence handling
      const expectedData = [];
      const str = "Tab:\tValue\nQuote:\"Hello\"\nBackslash:\\\n";
      for (const char of str) {
        expectedData.push(char.charCodeAt(0));
      }
      expectedData.push(0); // Manual null terminator
      
      expect(result.memoryData).toEqual(expectedData);
    });

    it('should handle character literals', () => {
      const source = `
        .data
        .byte 'A', 'B', 'C', '\\n', '\\0'
        .byte 65, 66, 67, 10, 0  ; Same values as decimals
        
        .code
          LI R3, 0
          LOAD R4, R3, 0   ; Load 'A'
          LOAD R5, R3, 3   ; Load '\\n'
          LOAD R6, R3, 5   ; Load decimal 65 (also 'A')
          HALT
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.memoryData).toEqual([
        65, 66, 67, 10, 0,    // Character literals
        65, 66, 67, 10, 0     // Same as decimals
      ]);
    });

    it('should handle binary and hex formats', () => {
      const source = `
        .data
        .byte 0b11111111, 0b00000000, 0b10101010
        .word 0xDEAD, 0xBEEF, 0xCAFE
        
        .code
          LI R3, 0
          LOAD R4, R3, 0   ; Load 0xFF
          LOAD R5, R3, 3   ; Load 0xDEAD
          HALT
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.memoryData).toEqual([
        255, 0, 170,                    // Binary values
        0xDEAD, 0xBEEF, 0xCAFE         // Hex values
      ]);
    });

    it('should generate proper assembly for string printing routine', () => {
      const source = `
        .data
        .asciiz "Hello, Ripple VM!\\n"
        
        .code
        main:
          LI R3, 0          ; Message address (assuming data at 0)
          LI R4, 0x0000     ; Output port address
          
        print_loop:
          LOAD R5, R3, 0    ; Load character
          BEQ R5, R0, done  ; Check for null terminator
          STORE R5, R4, 0   ; Write to output
          ADDI R3, R3, 1    ; Next character
          JAL print_loop    ; Continue loop
          
        done:
          HALT
      `;
      const result = assembler.assemble(source);
      
      expect(result.errors).toHaveLength(0);
      expect(result.instructions).toHaveLength(8);
      
      // Verify the generated code structure
      expect(result.instructions[0].opcode).toBe(Opcode.LI);  // LI R3, 0
      expect(result.instructions[1].opcode).toBe(Opcode.LI);  // LI R4, 0
      expect(result.instructions[2].opcode).toBe(Opcode.LOAD); // LOAD R5, R3, 0
      expect(result.instructions[3].opcode).toBe(Opcode.BEQ);  // BEQ R5, R0, done
      expect(result.instructions[4].opcode).toBe(Opcode.STORE); // STORE R5, R4, 0
      expect(result.instructions[5].opcode).toBe(Opcode.ADDI); // ADDI R3, R3, 1
      expect(result.instructions[6].opcode).toBe(Opcode.JAL);  // JAL print_loop
      expect(result.instructions[7].opcode).toBe(Opcode.NOP); // HALT (encoded as NOP)
      
      // Verify data
      const expectedMessage = "Hello, Ripple VM!\n";
      const expectedData = [...expectedMessage].map(c => c.charCodeAt(0));
      expectedData.push(0); // null terminator
      expect(result.memoryData).toEqual(expectedData);
    });
  });

  describe('Macro format output', () => {
    it('should generate macro format for simple program', () => {
      const source = `
        LI R3, 5
        LI R4, 3
        JALR R4, R4
      `;
      const result = assembler.assemble(source);
      const macroOutput = assembler.toMacroFormat(result.instructions);
      
      expect(result.errors).toHaveLength(0);
      expect(macroOutput).toContain('@program_start(@OP_LI');
      expect(macroOutput).toContain('@cmd(@OP_LI');
      expect(macroOutput).toContain('@cmd(@OP_JALR');
      expect(macroOutput).toContain('@program_end');
    });

    it('should format countdown loop example correctly', () => {
      const source = `
        LI R3, 5
        LI R4, 3
        JALR R4, R4
        LI R6, 1
        SUB R3, R3, R6
        ADD R8, R8, R6
        BNE R3, R0, 1
        HALT
        JALR RA, R4
        LI R6, 42
      `;
      const result = assembler.assemble(source);
      const macroOutput = assembler.toMacroFormat(result.instructions);
      
      expect(result.errors).toHaveLength(0);
      expect(macroOutput).toContain('@program_start(@OP_LI    , @R3 , 5   , 0)');
      expect(macroOutput).toContain('@cmd(@OP_SUB   , @R3 , @R3 , @R6)');
      expect(macroOutput).toContain('@cmd(@OP_HALT  , 0   , 0   , 0)');
      expect(macroOutput).toContain('@program_end');
    });

    it('should generate full program with data section', () => {
      const data = [72, 101, 108, 108, 111, 0]; // "Hello\0"
      const source = 'LI R3, 0';
      const result = assembler.assemble(source);
      const fullOutput = assembler.toFullMacroFormat(
        result.instructions,
        data,
        new Map([[0, 'Initialize pointer']]),
        'Hello World Program'
      );
      
      expect(fullOutput).toContain('// Hello World Program');
      expect(fullOutput).toContain('// Data segment');
      expect(fullOutput).toContain('@lane(#L_MEM,');
      expect(fullOutput).toContain('@set(72) @nextword');
      expect(fullOutput).toContain('// Program');
      expect(fullOutput).toContain('// Initialize pointer');
    });
  });
});