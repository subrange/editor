import { describe, it, expect } from 'vitest';
import { createMacroExpander } from './macro-expander';

describe('MacroExpander V2 - Validation Features', () => {
  describe('Macro definitions with leading whitespace', () => {
    it('should recognize macros with spaces before #define', () => {
      const input = `  #define test +\n@test`;
      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+');
    });

    it('should recognize macros with tabs before #define', () => {
      const input = `\t#define test -\n@test`;
      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('-');
    });
  });
  describe('Early validation of macro definitions', () => {
    it('should report undefined macro in definition immediately', () => {
      const input = `#define a @unknown`;
      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0]).toMatchObject({
        type: 'undefined',
        message: "Macro 'unknown' is not defined",
        location: {
          line: 0,
          column: 10,
          length: 8
        }
      });
    });

    it('should allow forward references between macros', () => {
      const input = `#define a @b
#define b +`;
      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.macros).toHaveLength(2);
    });

    it('should validate builtin functions in macro definitions', () => {
      const input = `#define bad {repeat(xyz, +)}`;
      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      // 'xyz' could be a parameter or future macro, so no error
      expect(result.errors).toHaveLength(0);
    });

    it('should not validate parameters as undefined', () => {
      const input = `#define inc(n) {repeat(n, +)}`;
      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
    });

    it('should validate nested macro invocations in definitions', () => {
      const input = `#define outer @inner(5)
#define inner(x) {repeat(x, +)}`;
      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
    });

    it('should report multiple undefined macros', () => {
      const input = `#define test @foo @bar @baz`;
      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(3);
      expect(result.errors.every(e => e.type === 'undefined')).toBe(true);
      expect(result.errors.map(e => e.message)).toEqual([
        "Macro 'foo' is not defined",
        "Macro 'bar' is not defined",
        "Macro 'baz' is not defined"
      ]);
    });
  });

  describe('Complex macro expansion with parameter substitution', () => {
    it('should substitute parameters in nested macro calls', () => {
      const input = `#define next(n) {repeat(n, >)}
#define L_SCRATCH_A 1
#define lane(n) @next(n)
#define lane_sA @lane(@L_SCRATCH_A)
@lane_sA`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('>');
    });

    it('should substitute parameters in builtin function arguments', () => {
      const input = `#define move(dir, count) {repeat(count, dir)}
@move(>, 3)`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('\n>>>');  // Default is collapseEmptyLines: false
    });

    it('should handle complex parameter substitution chains', () => {
      const input = `#define A 2
#define B @A
#define fn(x) {repeat(x, +)}
#define indirect(y) @fn(y)
@indirect(@B)`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++');
    });

    it('should substitute text parameters that look like identifiers', () => {
      const input = `#define wrapper(param) {repeat(param, -)}
@wrapper(5)`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('-----');
    });
  });

  describe('Conditional macro expansion', () => {
    it('should handle if builtin with macro parameters', () => {
      const input = `#define cond(x) {if(x, >, <)}
@cond(1)`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('\n>');  // Default is collapseEmptyLines: false
    });

    it('should handle if builtin with zero condition', () => {
      const input = `#define cond(x) {if(x, >, <)}
@cond(0)`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('\n<');  // Default is collapseEmptyLines: false
    });

    it('should handle nested if conditions', () => {
      const input = `#define A 1
#define B 0
#define test {if(@A, {if(@B, +, -)}, *)}
@test`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('\n\n\n-');  // Default is collapseEmptyLines: false
    });
  });

  describe('Error location reporting', () => {
    it('should report correct line and column for errors', () => {
      const input = `#define a +
#define b @undefined
#define c -`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0].location).toEqual({
        line: 1,  // Second line (0-indexed)
        column: 10,
        length: 10
      });
    });

    it('should report errors in multiline macros', () => {
      const input = `#define test \\
  @foo \\
  @bar
@test`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors.some(e => e.message.includes('foo'))).toBe(true);
      expect(result.errors.some(e => e.message.includes('bar'))).toBe(true);
    });
  });

  describe('Validation edge cases', () => {
    it('should not report errors for macros that are defined later in the same batch', () => {
      const input = `#define uses_later @defined_later
#define defined_later +
@uses_later`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+');
    });

    it('should handle circular references gracefully during validation', () => {
      const input = `#define a @b
#define b @c
#define c @a
@a`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      // Should not crash during validation, but will error during expansion
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors.some(e => e.type === 'circular_dependency')).toBe(true);
    });

    it('should validate macros with mixed valid and invalid references', () => {
      const input = `#define valid +
#define mixed @valid @invalid @valid
@mixed`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].message).toContain('invalid');
      expect(result.expanded).toContain('+ @invalid +');
    });

    it('should handle empty macro bodies', () => {
      const input = `#define empty
#define uses_empty @empty+@empty
@uses_empty`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+');
    });
  });

  describe('Builtin function validation', () => {
    it('should validate repeat with non-numeric literal', () => {
      const input = `#define bad {repeat(abc, +)}
@bad`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      // During definition, 'abc' could be a parameter, so no error
      // But during expansion, it will fail
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].message).toContain('Invalid repeat count: abc');
    });

    it('should validate if conditions properly', () => {
      const input = `#define test {if(not_a_number, +, -)}
@test`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].message).toContain('Invalid if condition');
    });

    it('should handle builtin functions with macro invocations as arguments', () => {
      const input = `#define num 5
#define test {repeat(@num, *)}
@test`;

      const expander = createMacroExpander();
      const result = expander.expand(input, { collapseEmptyLines: false });
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('*****');
    });

    it('should handle nested if builtins with parameter substitution', () => {
      const input = `#define test(lane, bit) {if(lane, {if(bit, +, -)}, {if(bit, <, >)})}
@test(0, 1)`;

      const expander = createMacroExpander();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('<');
    });
  });

  describe('Comment and whitespace handling', () => {
    it('should strip comments when requested', () => {
      const input = `// Comment before
#define test + // inline comment
/* multi
   line
   comment */
@test`;

      const expander = createMacroExpander();
      const result = expander.expand(input, { stripComments: true });
      
      expect(result.expanded).not.toContain('//');
      expect(result.expanded).not.toContain('/*');
      expect(result.expanded.trim()).toBe('+');
    });

    it('should preserve comments when requested', () => {
      const input = `// Comment
#define test +
@test // usage`;

      const expander = createMacroExpander();
      const result = expander.expand(input, { stripComments: false, collapseEmptyLines: false });
      
      expect(result.expanded).toContain('// Comment');
      expect(result.expanded).toContain('// usage');
    });

    it('should collapse empty lines when requested', () => {
      const input = `#define a +
#define b -

@a

@b`;

      const expander = createMacroExpander();
      const result = expander.expand(input, { collapseEmptyLines: true });
      
      // Should only have lines with BF commands
      const lines = result.expanded.split('\n').filter(l => l.trim());
      expect(lines).toEqual(['+', '-']);
    });
  });
});