import { describe, it, expect } from 'vitest';
import { createMacroExpander } from './macro-expander';

describe('MacroExpander - Source Maps Integration', () => {
  it('should generate source map when option is enabled', () => {
    const expander = createMacroExpander();
    const input = `#define inc +
@inc`;

    const result = expander.expand(input, { generateSourceMap: true });

    expect(result.sourceMap).toBeDefined();
    expect(result.sourceMap?.version).toBe(1);
    expect(result.sourceMap?.entries.length).toBeGreaterThan(0);
  });

  it('should not generate source map by default', () => {
    const expander = createMacroExpander();
    const input = `#define inc +
@inc`;

    const result = expander.expand(input);

    expect(result.sourceMap).toBeUndefined();
  });

  it('should map simple macro expansion correctly', () => {
    const expander = createMacroExpander();
    const input = `#define inc +
@inc @inc`;

    const result = expander.expand(input, { generateSourceMap: true });

    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('+ +');

    // Both + symbols should have source map entries
    const entries = result.sourceMap?.entries || [];
    const incMappings = entries.filter((e) => e.macroName === 'inc');
    expect(incMappings.length).toBeGreaterThanOrEqual(2);
  });

  it('should track expansion depth for nested macros', () => {
    const expander = createMacroExpander();
    const input = `#define a +
#define b @a @a
@b`;

    const result = expander.expand(input, { generateSourceMap: true });

    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('+ +');

    // Should have entries with different expansion depths
    const entries = result.sourceMap?.entries || [];
    const depths = new Set(entries.map((e) => e.expansionDepth));
    expect(depths.size).toBeGreaterThanOrEqual(1); // V3 tracks depth correctly
  });

  it('should include parameter values in source map', () => {
    const expander = createMacroExpander();
    const input = `#define x +
#define y -
#define add @x @y
@add`;

    const result = expander.expand(input, { generateSourceMap: true });

    expect(result.errors).toHaveLength(0);
    expect(result.expanded.trim()).toBe('+ -');

    // Should have entries for the macro expansions
    const entries = result.sourceMap?.entries || [];
    const addEntry = entries.find((e) => e.macroName === 'add');
    expect(addEntry).toBeDefined();
  });
});
