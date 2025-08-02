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

export interface MacroExpanderOptions {
  stripComments?: boolean;
  collapseEmptyLines?: boolean;
}

export interface MacroExpanderResult {
  expanded: string;
  errors: MacroExpansionError[];
  tokens: MacroToken[];
  macros: MacroDefinition[];
}

interface ParsedLine {
  type: 'definition' | 'code';
  content: string;
  indentation: string;
  lineNumber: number;
  sourceLineCount?: number; // Number of source lines this parsed line represents
}

export interface MacroExpander {
  expand(input: string, options?: MacroExpanderOptions): MacroExpanderResult;
}

export class MacroExpanderImpl implements MacroExpander {
  private macros: Map<string, MacroDefinition> = new Map();
  private expansionDepth = 0;
  private maxExpansionDepth = 100;
  private expansionChain: Set<string> = new Set();
  private errors: MacroExpansionError[] = [];
  private tokens: MacroToken[] = [];
  private currentOffset = 0;

  expand(input: string, options?: MacroExpanderOptions): MacroExpanderResult {
    const opts = {
      stripComments: true,
      collapseEmptyLines: true,
      ...options
    };

    this.macros.clear();
    this.expansionDepth = 0;
    this.expansionChain.clear();
    this.errors = [];
    this.tokens = [];
    this.currentOffset = 0;

    const lines = this.parseLines(input);
    this.collectMacros(lines);
    let expanded = this.expandLines(lines);

    if (opts.stripComments || opts.collapseEmptyLines) {
      expanded = this.postProcess(expanded, opts);
    }

    return {
      expanded,
      errors: this.errors,
      tokens: this.tokens,
      macros: Array.from(this.macros.values())
    };
  }

  private parseLines(input: string): ParsedLine[] {
    const lines = input.split('\n');
    const parsed: ParsedLine[] = [];

    let i = 0;
    while (i < lines.length) {
      const line = lines[i];
      const indentMatch = line.match(/^(\s*)/);
      const indentation = indentMatch ? indentMatch[1] : '';
      let trimmedLine = line.trim();
      const startLineNumber = i + 1;
      const startIndex = i;

      // Check for line continuation
      if (trimmedLine.startsWith('#define')) {
        // Collect all continued lines for macro definitions
        const continuedLines: string[] = [trimmedLine];
        
        while (i < lines.length - 1 && trimmedLine.endsWith('\\')) {
          // Remove the trailing backslash and any trailing spaces before it
          const lineWithoutBackslash = trimmedLine.slice(0, -1).trimEnd();
          continuedLines[continuedLines.length - 1] = lineWithoutBackslash;
          i++;
          const nextLine = lines[i];
          trimmedLine = nextLine.trim();
          continuedLines.push(trimmedLine);
        }

        // Join all continued lines with a space
        const fullContent = continuedLines.join(' ');
        
        parsed.push({
          type: 'definition',
          content: fullContent,
          indentation,
          lineNumber: startLineNumber,
          sourceLineCount: i - startIndex + 1
        });
      } else {
        parsed.push({
          type: 'code',
          content: line,
          indentation: '',
          lineNumber: startLineNumber,
          sourceLineCount: 1
        });
      }
      
      i++;
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
        // For definition lines, add empty lines for each source line consumed
        const sourceLineCount = line.sourceLineCount || 1;
        
        // Track the offset
        this.currentOffset += line.content.length + line.indentation.length + 1;
        
        // Add empty lines for each source line that was consumed by this definition
        for (let i = 0; i < sourceLineCount; i++) {
          result.push('');
        }
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
    // We need to manually parse to handle nested parentheses correctly
    let expandedText = '';
    let i = 0;
    
    while (i < result.length) {
      // Check for macro invocation
      if (result[i] === '@' && i + 1 < result.length && /\w/.test(result[i + 1])) {
        const startOffset = i;
        i++; // Skip @
        
        // Extract macro name
        let macroName = '';
        while (i < result.length && /\w/.test(result[i])) {
          macroName += result[i];
          i++;
        }
        
        // Check if it has parameters
        let args: string | undefined;
        let matchLength = 1 + macroName.length; // @ + name
        
        if (i < result.length && result[i] === '(') {
          // Extract parameters with proper parentheses matching
          let depth = 1;
          let argsStart = i + 1;
          i++; // Skip opening (
          
          while (i < result.length && depth > 0) {
            if (result[i] === '(') {
              depth++;
            } else if (result[i] === ')') {
              depth--;
            }
            i++;
          }
          
          if (depth === 0) {
            args = result.substring(argsStart, i - 1);
            matchLength = i - startOffset;
          } else {
            // Unclosed parentheses - treat as literal text
            expandedText += result.substring(startOffset, i);
            continue;
          }
        }
        
        // Process the macro
        const invocationStart = this.currentOffset + startOffset;
        
        if (!this.macros.has(macroName)) {
          this.errors.push({
            type: 'undefined',
            message: `Macro '${macroName}' is not defined`,
            location: lineNumber !== undefined ? {
              line: lineNumber - 1,
              column: startOffset,
              length: matchLength
            } : undefined
          });
          expandedText += result.substring(startOffset, startOffset + matchLength);
          continue;
        }

        this.tokens.push({
          type: 'macro_invocation',
          range: {
            start: invocationStart,
            end: invocationStart + matchLength
          },
          name: macroName
        });

        const macro = this.macros.get(macroName)!;
        
        if (args !== undefined) {
          // Parameterized macro invocation
          expandedText += this.expandParameterizedMacro(macro, args, lineNumber, startOffset, matchLength);
        } else {
          // Simple macro invocation
          expandedText += this.expandSimpleMacro(macro, lineNumber, startOffset, matchLength);
        }
      } else {
        expandedText += result[i];
        i++;
      }
    }

    return expandedText;
  }

  private expandBuiltins(text: string, lineNumber?: number): string {
    let result = text;
    
    // Expand repeat builtin
    const repeatRegex = /\{repeat\s*\(\s*(-?\d+)\s*,\s*([^)]+)\)}/g;
    result = result.replace(repeatRegex, (match, count, content, offset) => {
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
    
    // Expand if builtin
    // Syntax: {if(condition, true_branch, false_branch)}
    // condition: non-zero = true, zero = false
    // We need to manually parse to handle nested parentheses and commas properly
    let index = 0;
    while (true) {
      const ifMatch = result.substring(index).match(/\{if\s*\(/);
      if (!ifMatch) break;
      
      const offset = index + ifMatch.index!;
      const matchLength = ifMatch[0].length;
      const fullMatch = this.extractIfExpression(result, offset + matchLength);
      
      if (!fullMatch) {
        this.errors.push({
          type: 'syntax_error',
          message: `Invalid if expression`,
          location: lineNumber !== undefined ? {
            line: lineNumber - 1,
            column: offset,
            length: matchLength
          } : undefined
        });
        index = offset + matchLength;
        continue;
      }
      
      const args = this.parseArguments(fullMatch.content);
      
      if (args.length !== 3) {
        this.errors.push({
          type: 'syntax_error',
          message: `if() expects exactly 3 arguments, got ${args.length}`,
          location: lineNumber !== undefined ? {
            line: lineNumber - 1,
            column: offset,
            length: fullMatch.length
          } : undefined
        });
        index = offset + fullMatch.length;
        continue;
      }
      
      // Expand macros in the condition argument first
      const expandedCondition = this.expandMacrosInText(args[0], lineNumber);
      const condValue = parseInt(expandedCondition.trim(), 10);
      
      this.tokens.push({
        type: 'builtin_function',
        range: {
          start: this.currentOffset + offset,
          end: this.currentOffset + offset + fullMatch.length
        },
        name: 'if'
      });
      
      if (isNaN(condValue)) {
        this.errors.push({
          type: 'syntax_error',
          message: `Invalid if condition: ${args[0]} (expanded to: ${expandedCondition})`,
          location: lineNumber !== undefined ? {
            line: lineNumber - 1,
            column: offset,
            length: fullMatch.length
          } : undefined
        });
        index = offset + fullMatch.length;
        continue;
      }
      
      // Non-zero is true, zero is false
      // Also expand macros in the selected branch
      const selectedBranch = condValue !== 0 ? args[1] : args[2];
      const replacement = this.expandMacrosInText(selectedBranch, lineNumber);
      result = result.substring(0, offset) + replacement + result.substring(offset + fullMatch.length);
      index = offset + replacement.length;
    }
    
    return result;
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
      // Use a more flexible regex that matches the parameter name when it's:
      // - At a word boundary (for most cases)
      // - After @ (for macro invocations like @scratch_lane)
      // - After other non-word characters
      const regex = new RegExp(`(?<=^|[^\\w]|@)${param}(?=$|[^\\w])`, 'g');
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
    let i = 0;

    while (i < argsString.length) {
      const char = argsString[i];
      
      // Check for macro invocation
      if (char === '@' && i + 1 < argsString.length && /\w/.test(argsString[i + 1])) {
        // Found a macro invocation, consume the entire thing
        current += char;
        i++;
        
        // Consume the macro name
        while (i < argsString.length && /\w/.test(argsString[i])) {
          current += argsString[i];
          i++;
        }
        
        // Check if it has parameters
        if (i < argsString.length && argsString[i] === '(') {
          current += argsString[i];
          i++;
          let macroDepth = 1;
          
          // Consume everything until the matching closing parenthesis
          while (i < argsString.length && macroDepth > 0) {
            if (argsString[i] === '(') {
              macroDepth++;
            } else if (argsString[i] === ')') {
              macroDepth--;
            }
            current += argsString[i];
            i++;
          }
        }
        continue;
      }
      
      // Regular parentheses handling
      if (char === '(') {
        depth++;
        current += char;
        i++;
      } else if (char === ')') {
        depth--;
        current += char;
        i++;
      } else if (char === ',' && depth === 0) {
        // Only split on commas at depth 0
        args.push(current.trim());
        current = '';
        i++;
      } else {
        current += char;
        i++;
      }
    }

    if (current.trim()) {
      args.push(current.trim());
    }

    return args;
  }
  
  private extractIfExpression(text: string, startPos: number): { content: string; length: number } | null {
    // Start position is right after "{if("
    let depth = 1; // We're already inside one parenthesis
    let i = startPos;
    let content = '';
    const actualStart = text.lastIndexOf('{', startPos - 1); // Find the { before if(
    
    while (i < text.length && depth > 0) {
      const char = text[i];
      
      if (char === '(') {
        depth++;
      } else if (char === ')') {
        depth--;
        if (depth === 0) {
          // Found the closing parenthesis
          const closing = text[i + 1];
          if (closing === '}') {
            // Complete if expression found
            return {
              content,
              length: i + 2 - actualStart // From { to } inclusive
            };
          } else {
            // Missing closing brace
            return null;
          }
        }
      }
      
      if (depth > 0) {
        content += char;
      }
      i++;
    }
    
    // Unclosed parenthesis
    return null;
  }

  private postProcess(code: string, options: MacroExpanderOptions): string {
    let result = code;

    if (options.stripComments) {
      result = this.stripComments(result);
    }

    if (options.collapseEmptyLines) {
      result = this.collapseEmptyLines(result);
    }

    return result;
  }

  private stripComments(code: string): string {
    // First, remove all C-style comments /* */
    let result = code.replace(/\/\*[\s\S]*?\*\//g, '');
    
    // Then, remove all single-line comments //
    const lines = result.split('\n');
    const processedLines: string[] = [];
    
    for (const line of lines) {
      // Find the position of // comment
      const commentIndex = line.indexOf('//');
      let processedLine: string;
      
      if (commentIndex !== -1) {
        // Take only the part before the comment
        processedLine = line.substring(0, commentIndex);
      } else {
        processedLine = line;
      }
      
      processedLines.push(processedLine);
    }
    
    return processedLines.join('\n');
  }

  private collapseEmptyLines(code: string): string {
    const lines = code.split('\n');
    const nonEmptyLines: string[] = [];
    
    for (const line of lines) {
      // A line is considered non-empty if it has any BF commands (including $ for breakpoints)
      if (line.match(/[><+\-.\[\]$]/)) {
        nonEmptyLines.push(line);
      }
    }

    return nonEmptyLines.join('\n');
  }
}

// Import the new implementation
import { MacroExpanderV2 } from './macro-expander-v2.ts';

export function createMacroExpander(): MacroExpander {
  // Use the new lexer/parser-based implementation
  return new MacroExpanderV2();
}