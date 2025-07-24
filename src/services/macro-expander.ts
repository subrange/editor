export interface MacroDefinition {
  name: string;
  parameters?: string[];
  body: string;
  sourceLocation: {
    line: number;
    column: number;
    length: number;
  };
}

export interface MacroExpansionError {
  type: 'undefined' | 'parameter_mismatch' | 'circular_dependency' | 'syntax_error';
  message: string;
  location?: {
    line: number;
    column: number;
    length: number;
  };
}

export interface MacroToken {
  type: 'macro_definition' | 'macro_invocation' | 'builtin_function';
  range: {
    start: number;
    end: number;
  };
  name: string;
}

export interface MacroExpanderResult {
  expanded: string;
  errors: MacroExpansionError[];
  tokens: MacroToken[];
}

interface ParsedLine {
  type: 'definition' | 'code';
  content: string;
  indentation: string;
  lineNumber: number;
}

export interface MacroExpander {
  expand(input: string): MacroExpanderResult;
}

export class MacroExpanderImpl implements MacroExpander {
  private macros: Map<string, MacroDefinition> = new Map();
  private expansionDepth = 0;
  private maxExpansionDepth = 100;
  private expansionChain: Set<string> = new Set();
  private errors: MacroExpansionError[] = [];
  private tokens: MacroToken[] = [];
  private currentOffset = 0;

  expand(input: string): MacroExpanderResult {
    this.macros.clear();
    this.expansionDepth = 0;
    this.expansionChain.clear();
    this.errors = [];
    this.tokens = [];
    this.currentOffset = 0;

    const lines = this.parseLines(input);
    this.collectMacros(lines);
    const expanded = this.expandLines(lines);

    return {
      expanded,
      errors: this.errors,
      tokens: this.tokens
    };
  }

  private parseLines(input: string): ParsedLine[] {
    const lines = input.split('\n');
    const parsed: ParsedLine[] = [];

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const indentMatch = line.match(/^(\s*)/);
      const indentation = indentMatch ? indentMatch[1] : '';
      const trimmedLine = line.trim();

      if (trimmedLine.startsWith('#define')) {
        parsed.push({
          type: 'definition',
          content: trimmedLine,
          indentation,
          lineNumber: i + 1
        });
      } else {
        parsed.push({
          type: 'code',
          content: line,
          indentation: '',
          lineNumber: i + 1
        });
      }
    }

    return parsed;
  }

  private collectMacros(lines: ParsedLine[]): void {
    for (const line of lines) {
      if (line.type === 'definition') {
        const macro = this.parseMacroDefinition(line.content, line.lineNumber, line.indentation);
        if (macro) {
          if (this.macros.has(macro.name)) {
            this.errors.push({
              type: 'syntax_error',
              message: `Duplicate macro definition: '${macro.name}'`,
              location: {
                line: line.lineNumber - 1,
                column: line.indentation.length,
                length: line.content.length
              }
            });
          } else {
            this.macros.set(macro.name, macro);
          }
        }
      }
    }
  }

  private parseMacroDefinition(line: string, lineNumber: number, indentation: string): MacroDefinition | null {
    const simpleMatch = line.match(/^#define\s+(\w+)\s+(.*)$/);
    if (simpleMatch) {
      const startPos = this.currentOffset + indentation.length;
      this.tokens.push({
        type: 'macro_definition',
        range: {
          start: startPos,
          end: startPos + line.length
        },
        name: simpleMatch[1]
      });
      
      return {
        name: simpleMatch[1],
        body: simpleMatch[2].trim(),
        sourceLocation: {
          line: lineNumber - 1,
          column: indentation.length,
          length: line.length
        }
      };
    }

    const paramMatch = line.match(/^#define\s+(\w+)\s*\(([^)]+)\)\s+(.*)$/);
    if (paramMatch) {
      const params = paramMatch[2].split(',').map(p => p.trim());
      const startPos = this.currentOffset + indentation.length;
      this.tokens.push({
        type: 'macro_definition',
        range: {
          start: startPos,
          end: startPos + line.length
        },
        name: paramMatch[1]
      });
      
      return {
        name: paramMatch[1],
        parameters: params,
        body: paramMatch[3].trim(),
        sourceLocation: {
          line: lineNumber - 1,
          column: indentation.length,
          length: line.length
        }
      };
    }

    this.errors.push({
      type: 'syntax_error',
      message: `Invalid macro definition syntax`,
      location: {
        line: lineNumber - 1,
        column: indentation.length,
        length: line.length
      }
    });
    return null;
  }

  private expandLines(lines: ParsedLine[]): string {
    const result: string[] = [];

    for (const line of lines) {
      if (line.type === 'code') {
        result.push(this.expandLine(line.content, line.lineNumber));
      } else {
        // For definition lines, just track the offset
        this.currentOffset += line.content.length + line.indentation.length + 1;
        result.push(''); // Empty line for definitions
      }
    }

    return result.join('\n');
  }

  private expandLine(line: string, lineNumber: number): string {
    let expanded = line;
    let changed = true;
    let iterations = 0;
    const maxIterations = 100;

    while (changed && iterations < maxIterations) {
      const newExpanded = this.expandMacrosInText(expanded, lineNumber);
      changed = newExpanded !== expanded;
      expanded = newExpanded;
      iterations++;
    }

    if (iterations >= maxIterations) {
      this.errors.push({
        type: 'syntax_error',
        message: 'Maximum expansion iterations exceeded',
        location: {
          line: lineNumber - 1,
          column: 0,
          length: line.length
        }
      });
    }

    this.currentOffset += line.length + 1; // +1 for newline
    return expanded;
  }

  private expandMacrosInText(text: string, lineNumber?: number): string {
    let result = text;

    // First expand built-in functions
    result = this.expandBuiltins(result, lineNumber);

    // Then expand @-style macro invocations
    // Only match @ at word boundaries to avoid matching email-like patterns
    result = result.replace(/(?<!\w)@(\w+)(?:\((.*?)\))?/g, (match, macroName, args, offset) => {
      const invocationStart = this.currentOffset + offset;
      
      if (!this.macros.has(macroName)) {
        this.errors.push({
          type: 'undefined',
          message: `Macro '${macroName}' is not defined`,
          location: lineNumber !== undefined ? {
            line: lineNumber - 1,
            column: offset,
            length: match.length
          } : undefined
        });
        return match;
      }

      this.tokens.push({
        type: 'macro_invocation',
        range: {
          start: invocationStart,
          end: invocationStart + match.length
        },
        name: macroName
      });

      const macro = this.macros.get(macroName)!;
      
      if (args !== undefined) {
        // Parameterized macro invocation
        return this.expandParameterizedMacro(macro, args, lineNumber, offset, match.length);
      } else {
        // Simple macro invocation
        return this.expandSimpleMacro(macro, lineNumber, offset, match.length);
      }
    });

    return result;
  }

  private expandBuiltins(text: string, lineNumber?: number): string {
    const repeatRegex = /\{repeat\s*\(\s*(-?\d+)\s*,\s*([^)]+)\)}/g;
    return text.replace(repeatRegex, (match, count, content, offset) => {
      const n = parseInt(count, 10);
      
      this.tokens.push({
        type: 'builtin_function',
        range: {
          start: this.currentOffset + offset,
          end: this.currentOffset + offset + match.length
        },
        name: 'repeat'
      });
      
      if (isNaN(n) || n < 0) {
        this.errors.push({
          type: 'syntax_error',
          message: `Invalid repeat count: ${count}`,
          location: lineNumber !== undefined ? {
            line: lineNumber - 1,
            column: offset,
            length: match.length
          } : undefined
        });
        return match;
      }
      return content.repeat(n);
    });
  }

  private expandSimpleMacro(macro: MacroDefinition, lineNumber?: number, column?: number, length?: number): string {
    if (this.expansionChain.has(macro.name)) {
      const chain = Array.from(this.expansionChain).join(' → ');
      this.errors.push({
        type: 'circular_dependency',
        message: `Circular macro dependency detected: ${chain} → ${macro.name}`,
        location: lineNumber !== undefined && column !== undefined && length !== undefined ? {
          line: lineNumber - 1,
          column,
          length
        } : undefined
      });
      return `@${macro.name}`;
    }

    this.expansionDepth++;
    if (this.expansionDepth > this.maxExpansionDepth) {
      this.errors.push({
        type: 'syntax_error',
        message: `Maximum macro expansion depth exceeded`,
        location: lineNumber !== undefined && column !== undefined && length !== undefined ? {
          line: lineNumber - 1,
          column,
          length
        } : undefined
      });
      return `@${macro.name}`;
    }

    this.expansionChain.add(macro.name);
    const expanded = this.expandMacrosInText(macro.body);
    this.expansionChain.delete(macro.name);
    this.expansionDepth--;

    return expanded;
  }

  private expandParameterizedMacro(macro: MacroDefinition, argsString: string, lineNumber?: number, column?: number, length?: number): string {
    if (!macro.parameters) {
      this.errors.push({
        type: 'parameter_mismatch',
        message: `Macro '${macro.name}' does not accept parameters`,
        location: lineNumber !== undefined && column !== undefined && length !== undefined ? {
          line: lineNumber - 1,
          column,
          length
        } : undefined
      });
      return `@${macro.name}(${argsString})`;
    }

    const args = this.parseArguments(argsString);
    if (args.length !== macro.parameters.length) {
      this.errors.push({
        type: 'parameter_mismatch',
        message: `Macro '${macro.name}' expects ${macro.parameters.length} parameter(s), got ${args.length}`,
        location: lineNumber !== undefined && column !== undefined && length !== undefined ? {
          line: lineNumber - 1,
          column,
          length
        } : undefined
      });
      return `@${macro.name}(${argsString})`;
    }

    if (this.expansionChain.has(macro.name)) {
      const chain = Array.from(this.expansionChain).join(' → ');
      this.errors.push({
        type: 'circular_dependency',
        message: `Circular macro dependency detected: ${chain} → ${macro.name}`,
        location: lineNumber !== undefined && column !== undefined && length !== undefined ? {
          line: lineNumber - 1,
          column,
          length
        } : undefined
      });
      return `@${macro.name}(${argsString})`;
    }

    this.expansionDepth++;
    if (this.expansionDepth > this.maxExpansionDepth) {
      this.errors.push({
        type: 'syntax_error',
        message: `Maximum macro expansion depth exceeded`,
        location: lineNumber !== undefined && column !== undefined && length !== undefined ? {
          line: lineNumber - 1,
          column,
          length
        } : undefined
      });
      return `@${macro.name}(${argsString})`;
    }

    this.expansionChain.add(macro.name);

    let expandedBody = macro.body;
    for (let i = 0; i < macro.parameters.length; i++) {
      const param = macro.parameters[i];
      const arg = args[i];
      const regex = new RegExp(`\\b${param}\\b`, 'g');
      expandedBody = expandedBody.replace(regex, arg);
    }

    const result = this.expandMacrosInText(expandedBody);
    this.expansionChain.delete(macro.name);
    this.expansionDepth--;

    return result;
  }

  private parseArguments(argsString: string): string[] {
    const args: string[] = [];
    let current = '';
    let depth = 0;

    for (let i = 0; i < argsString.length; i++) {
      const char = argsString[i];
      
      if (char === '(') {
        depth++;
        current += char;
      } else if (char === ')') {
        depth--;
        current += char;
      } else if (char === ',' && depth === 0) {
        args.push(current.trim());
        current = '';
      } else {
        current += char;
      }
    }

    if (current.trim()) {
      args.push(current.trim());
    }

    return args;
  }
}

export function createMacroExpander(): MacroExpander {
  return new MacroExpanderImpl();
}