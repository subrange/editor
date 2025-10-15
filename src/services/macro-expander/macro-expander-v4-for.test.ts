import { describe, it, expect } from 'vitest';
import { createMacroExpanderV4 } from './macro-expander-v4.ts';

describe('Macro Expander V4 - For Loop', () => {
  it('should expand simple for loop', () => {
    const expander = createMacroExpanderV4();
    const input = `{for(x in {1, 2, 3}, x )}`;
    const result = expander.expand(input);

    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('1 2 3');
  });

  it('should expand for loop with text elements', () => {
    const expander = createMacroExpanderV4();
    const input = `{for(x in {a, b, c}, x )}`;
    const result = expander.expand(input);

    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('a b c');
  });

  it('should expand nested for loops', () => {
    const expander = createMacroExpanderV4();
    const input = `{for(i in {1, 2}, {for(j in {a, b}, i j )})}`;
    const result = expander.expand(input);

    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('1 a 1 b 2 a 2 b');
  });

  it('should work with for loop in macro', () => {
    const expander = createMacroExpanderV4();
    const input = `
      #define generatePairs(list1, list2) {
        {for(x in list1, {
          {for(y in list2, x y )}
        })}
      }
      @generatePairs({A, B}, {1, 2, 3})
    `;
    const result = expander.expand(input);

    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('A 1 A 2 A 3 B 1 B 2 B 3');
  });

  it('should handle for loop with complex body', () => {
    const expander = createMacroExpanderV4();
    const input = `
      #define processItem(item) {
        START item END
      }
      {for(x in {foo, bar, baz}, @processItem(x))}
    `;
    const result = expander.expand(input);

    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('STARTfooEND STARTbarEND STARTbazEND');
  });

  it('should handle empty array', () => {
    const expander = createMacroExpanderV4();
    const input = `{for(x in {}, x )}`;
    const result = expander.expand(input);

    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('');
  });

  it('should report error for invalid variable name', () => {
    const expander = createMacroExpanderV4();
    const input = `{for(123 in {1, 2, 3}, x )}`;
    const result = expander.expand(input);

    expect(result.errors).toHaveLength(2);
    expect(result.errors[0].message).toContain('variable name');
  });

  it('should report error for non-array second argument', () => {
    const expander = createMacroExpanderV4();
    const input = `{for(x in notArray, x )}`;
    const result = expander.expand(input);

    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].message).toContain('array');
  });

  it('should handle for loop with parameters in macro', () => {
    const expander = createMacroExpanderV4();
    const input = `
      #define iterate(arr, prefix) {
        {for(item in arr, prefix item )}
      }
      @iterate({X, Y, Z}, VALUE_)
    `;
    const result = expander.expand(input);

    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('VALUE_ X VALUE_ Y VALUE_ Z');
  });
});
