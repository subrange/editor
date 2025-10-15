import { describe, it, expect } from 'vitest';
import { SourceMapBuilder, SourceMapLookup } from './source-map';

describe('SourceMap', () => {
  describe('SourceMapBuilder', () => {
    it('should add and build mappings correctly', () => {
      const builder = new SourceMapBuilder();

      builder.addMapping({
        expandedRange: {
          start: { line: 1, column: 1 },
          end: { line: 1, column: 5 },
        },
        sourceRange: {
          start: { line: 2, column: 3 },
          end: { line: 2, column: 7 },
        },
        expansionDepth: 0,
      });

      const sourceMap = builder.build();

      expect(sourceMap.version).toBe(1);
      expect(sourceMap.entries).toHaveLength(1);
      // Now we have 2 entries: exact position and line-based
      expect(sourceMap.expandedToSource.size).toBe(2);
      expect(sourceMap.sourceToExpanded.size).toBe(2);
    });

    it('should handle multiple mappings at same position', () => {
      const builder = new SourceMapBuilder();

      // Two different expansions at the same source position
      builder.addMapping({
        expandedRange: {
          start: { line: 1, column: 1 },
          end: { line: 1, column: 5 },
        },
        sourceRange: {
          start: { line: 2, column: 3 },
          end: { line: 2, column: 7 },
        },
        expansionDepth: 0,
      });

      builder.addMapping({
        expandedRange: {
          start: { line: 2, column: 1 },
          end: { line: 2, column: 5 },
        },
        sourceRange: {
          start: { line: 2, column: 3 },
          end: { line: 2, column: 7 },
        },
        expansionDepth: 0,
      });

      const sourceMap = builder.build();
      const sourceEntries = sourceMap.sourceToExpanded.get('2:3');

      expect(sourceEntries).toHaveLength(2);
    });
  });

  describe('SourceMapLookup', () => {
    it('should find exact source position', () => {
      const builder = new SourceMapBuilder();

      builder.addMapping({
        expandedRange: {
          start: { line: 1, column: 1 },
          end: { line: 1, column: 5 },
        },
        sourceRange: {
          start: { line: 2, column: 3 },
          end: { line: 2, column: 7 },
        },
        expansionDepth: 0,
        macroName: 'testMacro',
      });

      const sourceMap = builder.build();
      const lookup = new SourceMapLookup(sourceMap);

      const entry = lookup.getSourcePosition(1, 1);

      expect(entry).not.toBeNull();
      expect(entry?.sourceRange.start.line).toBe(2);
      expect(entry?.sourceRange.start.column).toBe(3);
      expect(entry?.macroName).toBe('testMacro');
    });

    it('should find position within range', () => {
      const builder = new SourceMapBuilder();

      builder.addMapping({
        expandedRange: {
          start: { line: 1, column: 1 },
          end: { line: 1, column: 10 },
        },
        sourceRange: {
          start: { line: 2, column: 3 },
          end: { line: 2, column: 7 },
        },
        expansionDepth: 0,
      });

      const sourceMap = builder.build();
      const lookup = new SourceMapLookup(sourceMap);

      // Position in the middle of the range
      const entry = lookup.getSourcePosition(1, 5);

      expect(entry).not.toBeNull();
      expect(entry?.sourceRange.start.line).toBe(2);
      expect(entry?.sourceRange.start.column).toBe(3);
    });

    it('should prefer higher expansion depth', () => {
      const builder = new SourceMapBuilder();

      // Outer expansion
      builder.addMapping({
        expandedRange: {
          start: { line: 1, column: 1 },
          end: { line: 1, column: 20 },
        },
        sourceRange: {
          start: { line: 2, column: 1 },
          end: { line: 2, column: 10 },
        },
        expansionDepth: 0,
        macroName: 'outerMacro',
      });

      // Inner expansion (higher depth)
      builder.addMapping({
        expandedRange: {
          start: { line: 1, column: 5 },
          end: { line: 1, column: 10 },
        },
        sourceRange: {
          start: { line: 3, column: 1 },
          end: { line: 3, column: 5 },
        },
        expansionDepth: 1,
        macroName: 'innerMacro',
      });

      const sourceMap = builder.build();
      const lookup = new SourceMapLookup(sourceMap);

      // Position covered by both mappings
      const entry = lookup.getSourcePosition(1, 7);

      expect(entry).not.toBeNull();
      expect(entry?.macroName).toBe('innerMacro');
      expect(entry?.expansionDepth).toBe(1);
    });

    it('should get expanded positions for source', () => {
      const builder = new SourceMapBuilder();

      // Same source expands to multiple positions
      builder.addMapping({
        expandedRange: {
          start: { line: 1, column: 1 },
          end: { line: 1, column: 5 },
        },
        sourceRange: {
          start: { line: 2, column: 3 },
          end: { line: 2, column: 7 },
        },
        expansionDepth: 0,
      });

      builder.addMapping({
        expandedRange: {
          start: { line: 3, column: 1 },
          end: { line: 3, column: 5 },
        },
        sourceRange: {
          start: { line: 2, column: 3 },
          end: { line: 2, column: 7 },
        },
        expansionDepth: 0,
      });

      const sourceMap = builder.build();
      const lookup = new SourceMapLookup(sourceMap);

      const entries = lookup.getExpandedPositions(2, 3);

      expect(entries).toHaveLength(2);
      expect(entries[0].expandedRange.start.line).toBe(1);
      expect(entries[1].expandedRange.start.line).toBe(3);
    });

    it('should build macro context correctly', () => {
      const builder = new SourceMapBuilder();

      // Inner macro expansion (what we're looking for context of)
      builder.addMapping({
        expandedRange: {
          start: { line: 2, column: 5 },
          end: { line: 3, column: 10 },
        },
        sourceRange: {
          start: { line: 20, column: 1 },
          end: { line: 21, column: 10 },
        },
        expansionDepth: 1,
        macroName: 'innerMacro',
        macroCallSite: {
          start: { line: 10, column: 5 },
          end: { line: 10, column: 15 },
        },
        parameterValues: { x: '5' },
      });

      const sourceMap = builder.build();
      const lookup = new SourceMapLookup(sourceMap);

      // Just the inner macro context (no outer macro in this simple test)
      const context = lookup.getMacroContext(2, 6);

      expect(context).toHaveLength(1);
      expect(context[0].macroName).toBe('innerMacro');
      expect(context[0].parameterValues?.x).toBe('5');
    });
  });
});
