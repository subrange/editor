import { describe, it, expect } from 'vitest';
import { createMacroExpanderV3 } from './macro-expander-v3.ts';

describe('MacroExpander V3 - Hexadecimal Support', () => {
  describe('Basic hexadecimal parsing', () => {
    it('should parse hexadecimal numbers in repeat', () => {
      const expander = createMacroExpanderV3();
      const input = '{repeat(0x10, +)}';
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++++++++++++++++'); // 16 plus signs
    });

    it('should parse uppercase hexadecimal numbers', () => {
      const expander = createMacroExpanderV3();
      const input = '{repeat(0X0A, >)}';
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('>>>>>>>>>>'); // 10 right arrows
    });

    it('should parse large hexadecimal numbers', () => {
      const expander = createMacroExpanderV3();
      const input = '{repeat(0xFF, .)}';
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim().length).toBe(255); // 255 dots
    });

    it('should work in if statements', () => {
      const expander = createMacroExpanderV3();
      const input = '{if(0x01, +++, ---)}';
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+++');
    });

    it('should handle 0x00 as false in if statements', () => {
      const expander = createMacroExpanderV3();
      const input = '{if(0x00, +++, ---)}';
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('---');
    });
  });

  describe('Hexadecimal in macro definitions', () => {
    it('should work in macro invocations', () => {
      const expander = createMacroExpanderV3();
      const input = `#define add_n(n) {repeat(n, +)}
@add_n(0x20)`;
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim().length).toBe(32); // 32 plus signs
    });

    it('should handle hex values in macro definitions', () => {
      const expander = createMacroExpanderV3();
      const input = `#define BYTE_MAX {repeat(0xFF, +)}
@BYTE_MAX`;
      const result = expander.expand(input);
      
      if (result.errors.length > 0) {
        console.log('Errors:', result.errors);
        console.log('Expanded:', result.expanded);
      }
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim().length).toBe(255);
    });

    it('should work with parameter substitution', () => {
      const expander = createMacroExpanderV3();
      const input = `#define test(x) {repeat(x, >)}
@test(0x10)`;
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('>>>>>>>>>>>>>>>>'); // 16 right arrows
    });

    it('should work with macro references using @ syntax', () => {
      const expander = createMacroExpanderV3();
      const input = `#define BYTE_MAX 0xFF
#define fill_byte {repeat(@BYTE_MAX, +)}
@fill_byte`;
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim().length).toBe(255);
    });
  });

  describe('Hexadecimal in for loops', () => {
    it('should work in for loop arrays', () => {
      const expander = createMacroExpanderV3();
      const input = '{for(i in {0x01, 0x02, 0x03}, >)}';
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('>>>'); // 3 right arrows
    });
  });

  describe('Mixed case and formats', () => {
    it('should handle mixed case hex', () => {
      const expander = createMacroExpanderV3();
      const input = '{repeat(0xAb, -)}';
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim().length).toBe(171); // 0xAB = 171
    });

    it('should handle invalid hex gracefully', () => {
      const expander = createMacroExpanderV3();
      const input = '{repeat(0xZZ, +)}';
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0].type).toBe('syntax_error');
      expect(result.errors[0].message).toContain('Invalid repeat count');
    });
  });
});