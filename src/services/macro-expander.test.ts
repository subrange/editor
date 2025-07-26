import { describe, it, expect } from 'vitest';
import { createMacroExpander } from './macro-expander';

describe('MacroExpander', () => {
  const expander = createMacroExpander();

  describe('Simple macro expansion', () => {
    it('should expand a simple macro', () => {
      const input = `#define clear [-]
@clear`;
      const result = expander.expand(input, { collapseEmptyLines: false });
      expect(result.expanded).toBe('\n[-]');
      expect(result.errors).toHaveLength(0);
    });

    it('should expand multiple simple macros', () => {
      const input = `#define inc +
#define dec -
@inc @dec @inc`;
      const result = expander.expand(input);
      expect(result.expanded).toBe('\n\n+ - +');
      expect(result.errors).toHaveLength(0);
    });

    it('should preserve whitespace around macro invocations', () => {
      const input = `#define clear [-]
  @clear  @clear  `;
      const result = expander.expand(input);
      expect(result.expanded).toBe('\n  [-]  [-]  ');
      expect(result.errors).toHaveLength(0);
    });
  });

  describe('Parameterized macro expansion', () => {
    it('should expand a parameterized macro', () => {
      const input = `#define inc(n) {repeat(n, +)}
@inc(5)`;
      const result = expander.expand(input);
      expect(result.expanded).toBe('\n+++++');
      expect(result.errors).toHaveLength(0);
    });

    it('should expand macros with multiple parameters', () => {
      const input = `#define move(dir, n) {repeat(n, dir)}
@move(>, 3) @move(<, 2)`;
      const result = expander.expand(input);
      expect(result.expanded).toBe('\n>>> <<');
      expect(result.errors).toHaveLength(0);
    });

    it('should handle nested macro calls in arguments', () => {
      const input = `#define inc(n) {repeat(n, +)}
#define double(n) @inc(n) @inc(n)
@double(3)`;
      const result = expander.expand(input);
      expect(result.expanded).toBe('\n\n+++ +++');
      expect(result.errors).toHaveLength(0);
    });
  });

  describe('Nested macro expansion', () => {
    it('should expand nested macros', () => {
      const input = `#define clear [-]
#define clear2 @clear > @clear
@clear2`;
      const result = expander.expand(input);
      expect(result.expanded).toBe('\n\n[-] > [-]');
      expect(result.errors).toHaveLength(0);
    });

    it('should expand deeply nested macros', () => {
      const input = `#define a +
#define b @a @a
#define c @b @b
@c`;
      const result = expander.expand(input);
      expect(result.expanded).toBe('\n\n\n+ + + +');
      expect(result.errors).toHaveLength(0);
    });
  });

  describe('Built-in functions', () => {
    it('should expand repeat function', () => {
      const input = '{repeat(5, +)}';
      const result = expander.expand(input);
      expect(result.expanded).toBe('+++++');
      expect(result.errors).toHaveLength(0);
    });

    it('should handle repeat with complex content', () => {
      const input = '{repeat(3, >+<)}';
      const result = expander.expand(input);
      expect(result.expanded).toBe('>+<>+<>+<');
      expect(result.errors).toHaveLength(0);
    });

    it('should expand repeat in macro body', () => {
      const input = `#define inc(n) {repeat(n, +)}
@inc(4)`;
      const result = expander.expand(input);
      expect(result.expanded).toBe('\n++++');
      expect(result.errors).toHaveLength(0);
    });
  });

  describe('Error handling', () => {
    it('should report undefined macro error', () => {
      const input = '@undefined_macro';
      const result = expander.expand(input);
      expect(result.expanded).toBe('@undefined_macro');
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0]).toMatchObject({
        type: 'undefined',
        message: "Macro 'undefined_macro' is not defined"
      });
    });

    it('should report parameter mismatch - too few', () => {
      const input = `#define inc(n) {repeat(n, +)}
@inc()`;
      const result = expander.expand(input);
      expect(result.expanded).toBe('\n@inc()');
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0]).toMatchObject({
        type: 'parameter_mismatch',
        message: "Macro 'inc' expects 1 parameter(s), got 0"
      });
    });

    it('should report parameter mismatch - too many', () => {
      const input = `#define inc(n) {repeat(n, +)}
@inc(5, 10)`;
      const result = expander.expand(input);
      expect(result.expanded).toBe('\n@inc(5, 10)');
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0]).toMatchObject({
        type: 'parameter_mismatch',
        message: "Macro 'inc' expects 1 parameter(s), got 2"
      });
    });

    it('should report circular dependency', () => {
      const input = `#define a @b
#define b @a
@a`;
      const result = expander.expand(input);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors.some(e => e.type === 'circular_dependency')).toBe(true);
    });

    it('should report duplicate macro definition', () => {
      const input = `#define test +
#define test -`;
      const result = expander.expand(input);
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0]).toMatchObject({
        type: 'syntax_error',
        message: "Duplicate macro definition: 'test'"
      });
    });

    it('should report invalid repeat count', () => {
      const input = '{repeat(-5, +)}';
      const result = expander.expand(input);
      expect(result.expanded).toBe('{repeat(-5, +)}');
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0]).toMatchObject({
        type: 'syntax_error',
        message: 'Invalid repeat count: -5'
      });
    });
    
    it('should not expand invalid repeat syntax', () => {
      const input = '{repeat(abc, +)}';
      const result = expander.expand(input);
      expect(result.expanded).toBe('{repeat(abc, +)}');
      expect(result.errors).toHaveLength(0); // No error because it doesn't match the pattern
    });
  });

  describe('Edge cases', () => {
    it('should preserve @ symbol not followed by identifier', () => {
      const input = '@ @@ @ macro';
      const result = expander.expand(input);
      expect(result.expanded).toBe('@ @@ @ macro');
      expect(result.errors).toHaveLength(0);
    });

    it('should not expand email-like patterns', () => {
      const input = 'user@domain.com';
      const result = expander.expand(input);
      expect(result.expanded).toBe('user@domain.com');
      expect(result.errors).toHaveLength(0);
    });

    it('should handle @ with space after', () => {
      const input = `#define clear [-]
@ clear`;
      const result = expander.expand(input);
      expect(result.expanded).toBe('\n@ clear');
      expect(result.errors).toHaveLength(0);
    });

    it('should handle mixed macro and plain BF code', () => {
      const input = `#define inc(n) {repeat(n, +)}
#define dec(n) {repeat(n, -)}
>> @inc(5) << @dec(2)
+++[-]`;
      const result = expander.expand(input);
      expect(result.expanded).toBe('\n\n>> +++++ << --\n+++[-]');
      expect(result.errors).toHaveLength(0);
    });
  });

  describe('Token reporting', () => {
    it('should report macro definition tokens', () => {
      const input = `#define clear [-]
#define inc(n) {repeat(n, +)}`;
      const result = expander.expand(input);
      const defTokens = result.tokens.filter(t => t.type === 'macro_definition');
      expect(defTokens).toHaveLength(2);
      expect(defTokens[0].name).toBe('clear');
      expect(defTokens[1].name).toBe('inc');
    });

    it('should report macro invocation tokens', () => {
      const input = `#define clear [-]
@clear @clear`;
      const result = expander.expand(input);
      const invTokens = result.tokens.filter(t => t.type === 'macro_invocation');
      expect(invTokens).toHaveLength(2);
      expect(invTokens.every(t => t.name === 'clear')).toBe(true);
    });

    it('should report builtin function tokens', () => {
      const input = '{repeat(5, +)} {repeat(3, -)}';
      const result = expander.expand(input);
      const builtinTokens = result.tokens.filter(t => t.type === 'builtin_function');
      expect(builtinTokens).toHaveLength(2);
      expect(builtinTokens.every(t => t.name === 'repeat')).toBe(true);
    });
  });

  describe('Multiline macro definitions', () => {
    it('should expand a simple multiline macro', () => {
      const input = `#define hello_world ++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++. \\
>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>
@hello_world`;
      const result = expander.expand(input, { collapseEmptyLines: false });
      expect(result.expanded).toBe('\n\n++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++. >+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>');
      expect(result.errors).toHaveLength(0);
    });

    it('should expand a multiline parameterized macro', () => {
      const input = `#define copy(n) [-@next(n)+>+ \\
@prev(n)<]@next(n)[- \\
@prev(n)+@next(n)]@prev(n)
@copy(1)`;
      const result = expander.expand(input, { collapseEmptyLines: false });
      expect(result.expanded).toBe('\n\n\n[-@next(1)+>+ @prev(1)<]@next(1)[- @prev(1)+@next(1)]@prev(1)');
      expect(result.errors.length).toBeGreaterThan(0); // Will have errors for undefined @next and @prev
    });

    it('should handle multiple backslashes in a row', () => {
      const input = `#define multi_line \\
+ \\
+ \\
+
@multi_line`;
      const result = expander.expand(input, { collapseEmptyLines: false });
      expect(result.expanded).toBe('\n\n\n\n+ + +');
      expect(result.errors).toHaveLength(0);
    });

    it('should preserve spaces when joining continued lines', () => {
      const input = `#define spaced_macro >>  \\
  ++  \\
  <<
@spaced_macro`;
      const result = expander.expand(input, { collapseEmptyLines: false });
      expect(result.expanded).toBe('\n\n\n>> ++ <<');
      expect(result.errors).toHaveLength(0);
    });

    it('should handle backslash at end with continuation', () => {
      const input = `#define ends_with_backslash +++\\
@ends_with_backslash`;
      const result = expander.expand(input, { collapseEmptyLines: false });
      // The second line is treated as part of the macro body due to the backslash
      expect(result.expanded).toBe('\n');
      expect(result.errors).toHaveLength(0);
      // The macro body contains the invocation, creating a recursive definition
      expect(result.macros[0].body).toBe('+++ @ends_with_backslash');
    });

    it('should handle complex multiline macro with nested functions', () => {
      const input = `#define complex_macro(x, y) {repeat(x, +)} \\
> \\
{repeat(y, -)} \\
< \\
[-]
@complex_macro(3, 2)`;
      const result = expander.expand(input, { collapseEmptyLines: false });
      expect(result.expanded).toBe('\n\n\n\n\n+++ > -- < [-]');
      expect(result.errors).toHaveLength(0);
    });

    it('should handle multiline macro with comments', () => {
      const input = `#define with_comments // This is a comment \\
+ \\
// Another comment \\
+
@with_comments`;
      const result = expander.expand(input, { stripComments: false, collapseEmptyLines: false });
      expect(result.expanded).toBe('\n\n\n\n// This is a comment + // Another comment +');
      expect(result.errors).toHaveLength(0);
    });
  });

  describe('Complex real-world examples', () => {
    it('should handle the example from the task spec', () => {
      const input = `#define clear [-]
#define inc(n) {repeat(n, +)}
#define move(n) {repeat(n, >)}
#define clear2 @clear > @clear

// Using macros
@move(5) @inc(10) @clear

// Nested macro
@clear2

// Mixed with plain BF
>> @dec(2) <<< @inc(1)`;

      const result = expander.expand(input);
      
      // Note: @dec is undefined, so it should remain as-is
      expect(result.expanded).toContain('>>>>> ++++++++++ [-]');
      expect(result.expanded).toContain('[-] > [-]');
      expect(result.expanded).toContain('>> @dec(2) <<< +');
      
      // Should have error for undefined @dec
      expect(result.errors.some(e => 
        e.type === 'undefined' && e.message.includes('dec')
      )).toBe(true);
    });

    it('should handle a Brainfuck program with macros', () => {
      const input = `#define zero [-]
#define inc(n) {repeat(n, +)}
#define dec(n) {repeat(n, -)}
#define right(n) {repeat(n, >)}
#define left(n) {repeat(n, <)}
#define print .

// Print "Hi" (ASCII 72, 105)
@inc(72) @print
@right(1) @inc(105) @print
@left(1) @zero @right(1) @zero`;

      const result = expander.expand(input);
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toContain('++++++++++++'); // Should have 72 +'s
      expect(result.expanded).toContain('.');
    });
  });
});