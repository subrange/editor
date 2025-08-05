import { describe, it, expect } from 'vitest';
import { createMacroExpanderV3 } from './macro-expander.ts';

describe('MacroExpander V3 - Reverse Builtin Support', () => {
  describe('Basic reverse functionality', () => {
    it('should reverse an array literal', () => {
      const input = `{reverse({1, 2, 3})}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('{3, 2, 1}');
    });

    it('should reverse array with text values', () => {
      const input = `{reverse({a, b, c, d})}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('{d, c, b, a}');
    });

    it('should work with for loops', () => {
      const input = `{for(i in {reverse({1, 2, 3})}, i)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('321');
    });

    it('should reverse array from macro', () => {
      const input = `#define nums {1, 2, 3, 4, 5}
{reverse(@nums)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('{5, 4, 3, 2, 1}');
    });

    it('should handle empty array', () => {
      const input = `{reverse({})}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('{}');
    });

    it('should handle single element array', () => {
      const input = `{reverse({42})}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('{42}');
    });

    it('should report error for non-array argument', () => {
      const input = `{reverse(123)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].message).toContain('array literal');
    });

    it('should report error for wrong number of arguments', () => {
      const input = `{reverse({1, 2}, {3, 4})}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].message).toContain('expects exactly 1 argument');
    });

    it('should work in complex expressions', () => {
      const input = `#define process(arr) {for(x in {reverse(arr)}, @inc(x))}
#define inc(n) {repeat(n, +)}
@process({1, 2, 3})`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++++++'); // 3+2+1 = 6 pluses
    });
  });
});

describe('MacroExpander V3 - For Loop Support', () => {
  describe('Basic for loop functionality', () => {
    it('should expand for loop with array literal', () => {
      const input = `{for(i in {1, 2, 3}, +)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('+++');
    });

    it('should expand for loop with macro in body', () => {
      const input = `#define inc(n) {repeat(n, +)}
{for(v in {1, 2, 3}, @inc(v))}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++++++'); // 1 + 2 + 3 = 6 pluses
    });

    it('should expand for loop with complex body', () => {
      const input = `#define set(n) [-]{repeat(n, +)}
{for(v in {3, 5}, @set(v) >)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('[-]+++>[-]+++++>');
    });

    it('should handle for loop with macro that returns array', () => {
      const input = `#define nums {1, 2, 3, 4, 5}
{for(x in @nums, <)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('<<<<<');
    });

    it('should handle nested for loops', () => {
      const input = `{for(i in {1, 2}, {for(j in {3, 4}, i j)})}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('13142324');
    });

    it('should report error for invalid for syntax', () => {
      const input = `{for(123 in {1, 2, 3}, +)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].message).toContain('Expected variable name');
    });

    it('should report error for missing in keyword', () => {
      const input = `{for(i {1, 2, 3}, +)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].message).toContain('in');
    });

    it('should handle empty array in for loop', () => {
      const input = `{for(i in {}, +)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('');
    });
  });
});

describe('MacroExpander V3 - Validation Features', () => {
  describe('Macro definitions with leading whitespace', () => {
    it('should recognize macros with spaces before #define', () => {
      const input = `  #define test +\n@test`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+');
    });

    it('should recognize macros with tabs before #define', () => {
      const input = `\t#define test -\n@test`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('-');
    });
  });
  describe('Early validation of macro definitions', () => {
    it('should report undefined macro in definition immediately', () => {
      const input = `#define a @unknown`;
      const expander = createMacroExpanderV3();
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
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.macros).toHaveLength(2);
    });

    it('should validate builtin functions in macro definitions', () => {
      const input = `#define bad {repeat(xyz, +)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      // 'xyz' could be a parameter or future macro, so no error
      expect(result.errors).toHaveLength(0);
    });

    it('should not validate parameters as undefined', () => {
      const input = `#define inc(n) {repeat(n, +)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
    });

    it('should validate nested macro invocations in definitions', () => {
      const input = `#define outer @inner(5)
#define inner(x) {repeat(x, +)}`;
      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
    });

    it('should report multiple undefined macros', () => {
      const input = `#define test @foo @bar @baz`;
      const expander = createMacroExpanderV3();
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

      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('>');
    });

    it('should substitute parameters in builtin function arguments', () => {
      const input = `#define move(dir, count) {repeat(count, dir)}
@move(>, 3)`;

      const expander = createMacroExpanderV3();
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

      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('++');
    });

    it('should substitute text parameters that look like identifiers', () => {
      const input = `#define wrapper(param) {repeat(param, -)}
@wrapper(5)`;

      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('-----');
    });
  });

  describe('Conditional macro expansion', () => {
    it('should handle if builtin with macro parameters', () => {
      const input = `#define cond(x) {if(x, >, <)}
@cond(1)`;

      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('\n>');  // Default is collapseEmptyLines: false
    });

    it('should handle if builtin with zero condition', () => {
      const input = `#define cond(x) {if(x, >, <)}
@cond(0)`;

      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded).toBe('\n<');  // Default is collapseEmptyLines: false
    });

    it('should handle nested if conditions', () => {
      const input = `#define A 1
#define B 0
#define test {if(@A, {if(@B, +, -)}, *)}
@test`;

      const expander = createMacroExpanderV3();
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

      const expander = createMacroExpanderV3();
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

      const expander = createMacroExpanderV3();
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

      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+');
    });

    it('should handle circular references gracefully during validation', () => {
      const input = `#define a @b
#define b @c
#define c @a
@a`;

      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      // Should not crash during validation, but will error during expansion
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors.some(e => e.type === 'circular_dependency')).toBe(true);
    });

    it('should validate macros with mixed valid and invalid references', () => {
      const input = `#define valid +
#define mixed @valid @invalid @valid
@mixed`;

      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].message).toContain('invalid');
      expect(result.expanded).toContain('+ @invalid +');
    });

    it('should handle empty macro bodies', () => {
      const input = `#define empty
#define uses_empty @empty+@empty
@uses_empty`;

      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('+');
    });
  });

  describe('Builtin function validation', () => {
    it('should validate repeat with non-numeric literal', () => {
      const input = `#define bad {repeat(abc, +)}
@bad`;

      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      // During definition, 'abc' could be a parameter, so no error
      // But during expansion, it will fail
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].message).toContain('Invalid repeat count: abc');
    });

    it('should validate if conditions properly', () => {
      const input = `#define test {if(not_a_number, +, -)}
@test`;

      const expander = createMacroExpanderV3();
      const result = expander.expand(input);
      
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors[0].message).toContain('Invalid if condition');
    });

    it('should handle builtin functions with macro invocations as arguments', () => {
      const input = `#define num 5
#define test {repeat(@num, *)}
@test`;

      const expander = createMacroExpanderV3();
      const result = expander.expand(input, { collapseEmptyLines: false });
      
      expect(result.errors).toHaveLength(0);
      expect(result.expanded.trim()).toBe('*****');
    });

    it('should handle nested if builtins with parameter substitution', () => {
      const input = `#define test(lane, bit) {if(lane, {if(bit, +, -)}, {if(bit, <, >)})}
@test(0, 1)`;

      const expander = createMacroExpanderV3();
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

      const expander = createMacroExpanderV3();
      const result = expander.expand(input, { stripComments: true });
      
      expect(result.expanded).not.toContain('//');
      expect(result.expanded).not.toContain('/*');
      expect(result.expanded.trim()).toBe('+');
    });

    it('should preserve comments when requested', () => {
      const input = `// Comment
#define test +
@test // usage`;

      const expander = createMacroExpanderV3();
      const result = expander.expand(input, { stripComments: false, collapseEmptyLines: false });
      
      expect(result.expanded).toContain('// Comment');
      expect(result.expanded).toContain('// usage');
    });

    it('should collapse empty lines when requested', () => {
      const input = `#define a +
#define b -

@a

@b`;

      const expander = createMacroExpanderV3();
      const result = expander.expand(input, { collapseEmptyLines: true });
      
      // Should only have lines with BF commands
      const lines = result.expanded.split('\n').filter(l => l.trim());
      expect(lines).toEqual(['+', '-']);
    });
  });
});

describe('MacroExpander V3 - Nested Array Support', () => {
  it('should handle macros that expand to nested arrays in for loops', () => {
    const input = `#define PROGRAM {{1}, {2}}
#define set(v) +
{for(a in @PROGRAM, {for(v in a, @set(v))})}`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('++');
  });

  it('should handle nested arrays with multiple elements', () => {
    const input = `#define ARRAYS {{1, 2}, {3, 4, 5}}
#define process(x) {repeat(x, -)}
{for(arr in @ARRAYS, {for(val in arr, @process(val))})}`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('---------------'); // 1+2+3+4+5 = 15 dashes
  });

  it('should handle direct nested array literals in for loops', () => {
    const input = `{for(a in {{1, 2}, {3}}, {for(v in a, v)})}`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.errors).toHaveLength(0);
    expect(result.expanded).toBe('123');
  });
});

describe('MacroExpander V3 - Tuple Destructuring in For Loops', () => {
  it('should support tuple destructuring with two variables', () => {
    const input = `#define PROGRAM {1, 2}
#define set(a) +
#define next(b) >
{for((a, b) in {@PROGRAM}, @set(a) @next(b))}`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('+>');
  });

  it('should iterate over array of tuples', () => {
    const input = `#define PAIRS {{1, 2}, {3, 4}, {5, 6}}
#define process(x, y) {repeat(x, +)}{repeat(y, -)}
{for((a, b) in @PAIRS, @process(a, b))}`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('+--+++----+++++------');
  });

  it('should support tuple destructuring with three or more variables', () => {
    const input = `#define TRIPLES {{1, 2, 3}, {4, 5, 6}}
{for((x, y, z) in @TRIPLES, xyz)}`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('123456');
  });

  it('should handle direct tuple literals', () => {
    const input = `{for((a, b) in {{10, 20}, {30, 40}}, a-b)}`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.errors).toHaveLength(0);
    expect(result.expanded).toBe('10-2030-40');
  });

  it('should handle tuples with missing elements gracefully', () => {
    const input = `#define MIXED {{1}, {2, 3}, {4, 5, 6}}
{for((a, b, c) in @MIXED, abc)}`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('123456');
  });

  it('should work with nested for loops and tuples', () => {
    const input = `#define OUTER {{1, 2}, {3, 4}}
#define INNER {a, b}
{for((x, y) in @OUTER, {for(z in @INNER, xyz)})}`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.errors).toHaveLength(0);
    // When (x,y) = (1,2), we iterate z over {a,b} giving: 12a, 12b
    // When (x,y) = (3,4), we iterate z over {a,b} giving: 34a, 34b
    expect(result.expanded.trim()).toBe('12a12b34a34b');
  });
});

describe('MacroExpander V3 - Nested Array and Tuple Iteration', () => {
  it('should handle nested for loops with array of tuples', () => {
    const input = `#define T_OP 1
#define OP_NOP 0
#define fillword(type, val, flf) {repeat(type, +)}{repeat(val, -)}{repeat(flf, >)}

#define PROGRAM {
{@T_OP, @OP_NOP, 1}, 
{@T_OP, @OP_NOP, 2}
}

{for(a in @PROGRAM, {for((type, val, flf) in {a}, @fillword(type, val, flf))})}`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.errors).toHaveLength(0);
    // First tuple: type=1, val=0, flf=1 -> +>
    // Second tuple: type=1, val=0, flf=2 -> +>>
    expect(result.expanded.trim()).toBe('+>+>>');
  });

  it('should handle direct nested iteration', () => {
    const input = `#define show(x, y) x:y
{for(item in {{a, b}, {c, d}}, {for((x, y) in {item}, @show(x, y))})}`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('a:bc:d');
  });

  it('should work with complex nested structures', () => {
    const input = `#define DATA {
{1, 2, 3},
{4, 5, 6},
{7, 8, 9}
}
#define process(a, b, c) [abc]
{for(row in @DATA, {for((a, b, c) in {row}, @process(a, b, c))})}`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('[123][456][789]');
  });
});

describe('MacroExpander V3 - Source Map Support', () => {
  it('should generate source maps when requested', () => {
    const input = `#define inc(n) {repeat(n, +)}
@inc(3)`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input, { generateSourceMap: true });
    
    expect(result.errors).toHaveLength(0);
    expect(result.sourceMap).toBeDefined();
    expect(result.sourceMap!.entries.length).toBeGreaterThan(0);
  });

  it('should not generate source maps by default', () => {
    const input = `#define test +
@test`;
    
    const expander = createMacroExpanderV3();
    const result = expander.expand(input);
    
    expect(result.sourceMap).toBeUndefined();
  });
});