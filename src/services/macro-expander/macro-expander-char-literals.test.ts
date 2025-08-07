import { describe, it, expect } from 'vitest';
import { createMacroExpanderV3 } from './macro-expander.ts';

describe('MacroExpander V3 - Character Literal Support', () => {
  describe('Basic character literal functionality', () => {
    it('should expand character literal in repeat', () => {
      const input = `{repeat('A', +)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // 'A' = ASCII 65, so repeat 65 times
      expect(result.expanded).toBe('+'.repeat(65));
    });

    it('should support lowercase letters', () => {
      const input = `{repeat('a', -)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // 'a' = ASCII 97
      expect(result.expanded).toBe('-'.repeat(97));
    });

    it('should support digits as character literals', () => {
      const input = `{repeat('0', >)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // '0' = ASCII 48
      expect(result.expanded).toBe('>'.repeat(48));
    });

    it('should support special characters', () => {
      const input = `{repeat(' ', <)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // ' ' = ASCII 32
      expect(result.expanded).toBe('<'.repeat(32));
    });

    it('should work with if statements', () => {
      const input = `{if('A', YES, NO)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // 'A' = 65, which is truthy
      expect(result.expanded).toBe('YES');
    });

    it('should handle null character', () => {
      const input = `{if('\0', YES, NO)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // '\0' = 0, which is falsy
      expect(result.expanded).toBe('NO');
    });
  });

  describe('Character literals in macros', () => {
    it('should work as macro parameters', () => {
      const input = `#define printChar(n) {repeat(n, .)}
@printChar('H')`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // 'H' = ASCII 72
      expect(result.expanded.trim()).toBe('.'.repeat(72));
    });

    it('should work with hash invocation', () => {
      const input = `#define printChar(n) {repeat(n, .)}
#printChar('!')`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // '!' = ASCII 33
      expect(result.expanded.trim()).toBe('.'.repeat(33));
    });

    it('should support character literals in macro bodies', () => {
      const input = String.raw`#define newline {repeat('\n', .)}` + '\n@newline';
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // '\n' = ASCII 10
      expect(result.expanded.trim()).toBe('.'.repeat(10));
    });
  });

  describe('Character literals in arrays and loops', () => {
    it('should work in array literals', () => {
      const input = `{for(ch in {'A', 'B', 'C'}, {repeat(ch, +)})}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // 'A'=65, 'B'=66, 'C'=67
      expect(result.expanded).toBe('+'.repeat(65) + '+'.repeat(66) + '+'.repeat(67));
    });

    it('should work with reverse', () => {
      const input = `{reverse({'a', 'b', 'c'})}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // Note: Character literals are converted to their ASCII values during expansion
      // 'a'=97, 'b'=98, 'c'=99
      expect(result.expanded).toBe('{99, 98, 97}');
    });
  });

  describe('Mixed literals', () => {
    it('should work alongside decimal numbers', () => {
      const input = `{repeat('A', +)}{repeat(10, -)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('+'.repeat(65) + '-'.repeat(10));
    });

    it('should work alongside hex numbers', () => {
      const input = `{repeat('A', +)}{repeat(0x10, -)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('+'.repeat(65) + '-'.repeat(16));
    });

    it('should work in complex expressions', () => {
      const input = `#define ASCII(ch) {repeat(ch, +)}
#define HEX(n) {repeat(n, -)}
@ASCII('Z')
@HEX(0xFF)`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // 'Z' = 90, 0xFF = 255
      expect(result.expanded.trim()).toBe('+'.repeat(90) + '\n' + '-'.repeat(255));
    });
  });

  describe('Edge cases and error handling', () => {
    it('should handle escape sequences in character literals', () => {
      const input = String.raw`{repeat('\n', .)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // '\n' = newline character = ASCII 10
      expect(result.expanded).toBe('.'.repeat(10));
    });

    it('should work with punctuation characters', () => {
      const input = `{repeat('.', >)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // '.' = ASCII 46
      expect(result.expanded).toBe('>'.repeat(46));
    });

    it('should handle quotes inside character literals', () => {
      const input = `{repeat('"', <)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      // '"' = ASCII 34
      expect(result.expanded).toBe('<'.repeat(34));
    });
  });
});