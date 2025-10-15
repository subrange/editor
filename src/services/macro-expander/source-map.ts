export interface Position {
  line: number;
  column: number;
  offset?: number;
}

export interface Range {
  start: Position;
  end: Position;
}

export interface SourceMapEntry {
  // Position in expanded code
  expandedRange: Range;

  // Position in source code
  sourceRange: Range;

  // Macro context
  macroName?: string;
  macroCallSite?: Range;
  expansionDepth: number;
  parameterValues?: Record<string, string>;

  // Complete macro call stack at this position
  macroCallStack?: Array<{
    macroName: string;
    callSite: Range;
    parameters?: Record<string, string>;
  }>;
}

export interface SourceMap {
  version: 1;
  entries: SourceMapEntry[];

  // Optimized lookup structures
  expandedToSource: Map<string, SourceMapEntry[]>; // key: "line:column"
  sourceToExpanded: Map<string, SourceMapEntry[]>; // key: "line:column"
}

export class SourceMapBuilder {
  private entries: SourceMapEntry[] = [];
  private expandedToSource = new Map<string, SourceMapEntry[]>();
  private sourceToExpanded = new Map<string, SourceMapEntry[]>();

  addMapping(entry: SourceMapEntry): void {
    this.entries.push(entry);

    // Add to expanded lookup
    const expandedKey = `${entry.expandedRange.start.line}:${entry.expandedRange.start.column}`;
    const expandedEntries = this.expandedToSource.get(expandedKey) || [];
    expandedEntries.push(entry);
    this.expandedToSource.set(expandedKey, expandedEntries);

    // Also index by line for range-based lookups
    const expandedLineKey = `line:${entry.expandedRange.start.line}`;
    const expandedLineEntries =
      this.expandedToSource.get(expandedLineKey) || [];
    expandedLineEntries.push(entry);
    this.expandedToSource.set(expandedLineKey, expandedLineEntries);

    // Add to source lookup - index by start position
    const sourceKey = `${entry.sourceRange.start.line}:${entry.sourceRange.start.column}`;
    const sourceEntries = this.sourceToExpanded.get(sourceKey) || [];
    sourceEntries.push(entry);
    this.sourceToExpanded.set(sourceKey, sourceEntries);

    // Also index by line for range-based lookups
    const sourceLineKey = `line:${entry.sourceRange.start.line}`;
    const sourceLineEntries = this.sourceToExpanded.get(sourceLineKey) || [];
    sourceLineEntries.push(entry);
    this.sourceToExpanded.set(sourceLineKey, sourceLineEntries);
  }

  build(): SourceMap {
    return {
      version: 1,
      entries: this.entries,
      expandedToSource: this.expandedToSource,
      sourceToExpanded: this.sourceToExpanded,
    };
  }
}

export class SourceMapLookup {
  constructor(private sourceMap: SourceMap) {}

  getSourcePosition(
    expandedLine: number,
    expandedColumn: number,
  ): SourceMapEntry | null {
    // First try exact match
    const exactKey = `${expandedLine}:${expandedColumn}`;
    const exactEntries = this.sourceMap.expandedToSource.get(exactKey);
    console.log(
      `getSourcePosition: Looking for [${expandedLine}:${expandedColumn}], found ${exactEntries?.length || 0} exact entries`,
    );

    if (exactEntries && exactEntries.length > 0) {
      // Debug: Show all entries at this position
      exactEntries.forEach((e, i) => {
        console.log(
          `  Entry ${i}: ${e.macroName} depth=${e.expansionDepth} hasCallStack=${!!e.macroCallStack}`,
        );
      });

      // Return the entry with highest expansion depth (innermost)
      const result = exactEntries.reduce((prev, curr) =>
        curr.expansionDepth > prev.expansionDepth ? curr : prev,
      );
      console.log(
        `  Returning: ${result.macroName} depth=${result.expansionDepth}`,
      );
      return result;
    }

    // If no exact match, check line entries
    const lineKey = `line:${expandedLine}`;
    const lineEntries = this.sourceMap.expandedToSource.get(lineKey) || [];

    // Find entries that contain this position
    const matchingEntries = lineEntries.filter((entry) =>
      this.positionInRange(expandedLine, expandedColumn, entry.expandedRange),
    );

    if (matchingEntries.length > 0) {
      // Return the entry with highest expansion depth (innermost)
      return matchingEntries.reduce((prev, curr) =>
        curr.expansionDepth > prev.expansionDepth ? curr : prev,
      );
    }

    // Fall back to searching all entries (for backwards compatibility)
    let bestEntry: SourceMapEntry | null = null;
    let bestDistance = Infinity;

    for (const entry of this.sourceMap.entries) {
      if (
        this.positionInRange(expandedLine, expandedColumn, entry.expandedRange)
      ) {
        const distance = this.distanceToStart(
          expandedLine,
          expandedColumn,
          entry.expandedRange,
        );
        if (distance < bestDistance) {
          bestDistance = distance;
          bestEntry = entry;
        }
      }
    }

    return bestEntry;
  }

  getExpandedPositions(
    sourceLine: number,
    sourceColumn: number,
  ): SourceMapEntry[] {
    // First try exact match
    const exactKey = `${sourceLine}:${sourceColumn}`;
    const exactEntries = this.sourceMap.sourceToExpanded.get(exactKey);
    if (exactEntries && exactEntries.length > 0) {
      return exactEntries;
    }

    // If no exact match, find entries that contain this position
    const lineKey = `line:${sourceLine}`;
    const lineEntries = this.sourceMap.sourceToExpanded.get(lineKey) || [];

    // Filter entries that contain the given position within their source range
    const matchingEntries = lineEntries.filter((entry) =>
      this.positionInRange(sourceLine, sourceColumn, entry.sourceRange),
    );

    return matchingEntries;
  }

  getMacroContext(
    expandedLine: number,
    expandedColumn: number,
  ): SourceMapEntry[] {
    // Get the full macro expansion stack at a position
    const entry = this.getSourcePosition(expandedLine, expandedColumn);

    console.log(
      'getMacroContext: entry =',
      entry
        ? {
            macroName: entry.macroName,
            hasCallStack: !!entry.macroCallStack,
            callStackLength: entry.macroCallStack?.length || 0,
          }
        : 'null',
    );

    if (entry && entry.macroCallStack && entry.macroCallStack.length > 0) {
      // Return the stored call stack as proper entries
      const result: SourceMapEntry[] = [];

      // Add each level of the call stack
      entry.macroCallStack.forEach((stackEntry, index) => {
        result.push({
          expandedRange: entry.expandedRange,
          sourceRange: stackEntry.callSite,
          macroName: stackEntry.macroName || '',
          parameterValues: stackEntry.parameters,
          expansionDepth: index + 1,
          macroCallStack: entry.macroCallStack,
        });
      });

      // Note: Don't add the current entry again - it should already be in the call stack
      // The last entry in the call stack should be the current macro

      console.log(
        'getMacroContext: returning',
        result.length,
        'entries from call stack',
      );
      return result;
    }

    // Fallback for entries without call stack
    if (entry && entry.macroName) {
      console.log('getMacroContext: returning single entry (no call stack)');
      return [entry];
    }

    return [];
  }

  private positionInRange(line: number, column: number, range: Range): boolean {
    if (line < range.start.line || line > range.end.line) {
      return false;
    }

    if (line === range.start.line && column < range.start.column) {
      return false;
    }

    if (line === range.end.line && column >= range.end.column) {
      return false;
    }

    return true;
  }

  private distanceToStart(line: number, column: number, range: Range): number {
    const lineDiff = line - range.start.line;
    const colDiff = column - range.start.column;
    return lineDiff * 1000 + colDiff; // Weight lines more than columns
  }
}
