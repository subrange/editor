import { describe, it, expect } from 'vitest';
import { createMacroExpanderV3 } from './macro-expander-v3.ts';

describe('MacroExpander V3 - Hash Macro Invocation Support', () => {
  describe('Basic hash macro invocation', () => {
    it('should expand macros invoked with # prefix', () => {
      const expander = createMacroExpanderV3();
      const input = `#define add5 +++++
#add5`;
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+++++');
    });

    it('should expand parameterized macros with # prefix', () => {
      const expander = createMacroExpanderV3();
      const input = `#define repeat_n(n, cmd) {repeat(n, cmd)}
#repeat_n(3, >)`;
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('>>>');
    });

    it('should work with both @ and # in the same file', () => {
      const expander = createMacroExpanderV3();
      const input = `#define add3 +++
#define add5 +++++
@add3
#add5`;
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+++\n+++++');
    });

    it('should handle # prefix in nested macro calls', () => {
      const expander = createMacroExpanderV3();
      const input = `#define inner ++
#define outer #inner#inner
#outer`;
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++++');
    });

    it('should work with # prefix in builtin functions', () => {
      const expander = createMacroExpanderV3();
      const input = `#define move_right >
{repeat(3, #move_right)}`;
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('>>>');
    });

    it('should handle # prefix with hexadecimal numbers', () => {
      const expander = createMacroExpanderV3();
      const input = `#define hex_repeat(n, cmd) {repeat(n, cmd)}
#hex_repeat(0x10, +)`;
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim().length).toBe(16);
    });

    it('should error on undefined macros with # prefix', () => {
      const expander = createMacroExpanderV3();
      const input = `#undefined_macro`;
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(1);
      expect(result.errors[0].type).toBe('undefined');
      expect(result.errors[0].message).toContain('undefined_macro');
    });

    it('should handle # in for loops', () => {
      const expander = createMacroExpanderV3();
      const input = `#define add_one +
{for(i in {1, 2, 3}, #add_one)}`;
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+++');
    });
  });

  describe('Edge cases', () => {
    it('should handle # followed by non-identifier', () => {
      const expander = createMacroExpanderV3();
      const input = `# This is just text`;
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('# This is just text');
    });

    it('should not confuse #define with #macro', () => {
      const expander = createMacroExpanderV3();
      const input = `#define test +++
#test
#define another ---
#another`;
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+++\n\n---');
    });

    it('should handle mixed @ and # with same macro', () => {
      const expander = createMacroExpanderV3();
      const input = `#define test +++
@test#test`;
      const result = expander.expand(input);

      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++++++');
    });
  });
});
