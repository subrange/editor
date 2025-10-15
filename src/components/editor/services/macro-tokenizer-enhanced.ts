import { type ITokenizer } from '../../../services/editor-manager.service.ts';
import {
  type MacroToken as ExpanderToken,
  type MacroExpansionError,
  type MacroDefinition,
} from '../../../services/macro-expander/macro-expander.ts';
import { createAsyncMacroExpander } from '../../../services/macro-expander/create-macro-expander.ts';
import { type MacroExpanderWorkerClient } from '../../../services/macro-expander/macro-expander-worker-client.ts';

// Token types for macro syntax
export interface MacroToken {
  type:
    | 'macro'
    | 'macro_definition'
    | 'macro_invocation'
    | 'builtin_function'
    | 'parameter'
    | 'incdec'
    | 'brackets'
    | 'move'
    | 'dot'
    | 'comma'
    | 'whitespace'
    | 'comment'
    | 'todo_comment'
    | 'mark_comment'
    | 'unknown'
    | 'error'
    | 'parentheses'
    | 'braces'
    | 'macro_name'
    | 'number'
    | 'continuation'
    | 'keyword';
  value: string;
  start: number;
  end: number;
  error?: MacroExpansionError;
}

interface MacroTokenizerState {
  inMultiLineComment: boolean;
  expanderTokens: ExpanderToken[];
  expanderErrors: MacroExpansionError[];
  macroDefinitions: MacroDefinition[];
  expectMacroName?: boolean; // Track if we just saw #define
  inMacroDefinition?: boolean; // Track if we're in a macro definition line
  currentLineParams?: Set<string>; // Parameters for current macro definition line
  continuedMacroDefinition?: boolean; // Track if previous line ended with backslash
  inBraceMacroDefinition?: boolean; // Track if we're inside a brace-based macro definition
  macroDefinitionBraceDepth?: number; // Track brace depth for macro definition
  braceMacroParams?: Set<string>; // Parameters for brace-based macro definition
  forLoopVariables: Set<string>; // Track loop variables from {for} constructs
  braceDepth: number; // Track nesting depth to manage scope
  forLoopScopes: Array<{ variable: string; depth: number }>; // Stack of for loop scopes
}

export class EnhancedMacroTokenizer implements ITokenizer {
  public state: MacroTokenizerState = {
    inMultiLineComment: false,
    expanderTokens: [],
    expanderErrors: [],
    macroDefinitions: [],
    forLoopVariables: new Set(),
    braceDepth: 0,
    forLoopScopes: [],
    inBraceMacroDefinition: false,
    macroDefinitionBraceDepth: 0,
  };

  private asyncExpander: MacroExpanderWorkerClient | null = null;
  private fullText = '';
  private lineOffsets: number[] = [];
  private lastExpandPromise: Promise<void> | null = null;
  private stateChangeCallbacks: Set<() => void> = new Set();
  private lastProcessedText = '';

  reset() {
    this.state = {
      inMultiLineComment: false,
      expanderTokens: [],
      expanderErrors: [],
      macroDefinitions: [],
      continuedMacroDefinition: false,
      forLoopVariables: new Set(),
      braceDepth: 0,
      forLoopScopes: [],
      inBraceMacroDefinition: false,
      macroDefinitionBraceDepth: 0,
    };
    this.fullText = '';
    this.lineOffsets = [];
    this.lastProcessedText = '';
  }

  // Convert global position to line/column
  // private positionToLineColumn(position: number): { line: number, column: number } {
  //     for (let i = 0; i < this.lineOffsets.length - 1; i++) {
  //         if (position >= this.lineOffsets[i] && position < this.lineOffsets[i + 1]) {
  //             return { line: i, column: position - this.lineOffsets[i] };
  //         }
  //     }
  //     // Last line
  //     const lastLine = this.lineOffsets.length - 1;
  //     return { line: lastLine, column: position - this.lineOffsets[lastLine] };
  // }

  // Check if a position on a line has an error
  private findError(
    lineIndex: number,
    start: number,
    end: number,
  ): MacroExpansionError | undefined {
    return this.state.expanderErrors.find((error) => {
      if (!error.location) return false;
      return (
        error.location.line === lineIndex &&
        error.location.column < end &&
        error.location.column + error.location.length > start
      );
    });
  }

  tokenizeLine(
    text: string,
    lineIndex: number,
    _isLastLine: boolean = false,
  ): MacroToken[] {
    const tokens: MacroToken[] = [];
    let position = 0;

    // Check if we're inside a brace-based macro definition
    if (this.state.inBraceMacroDefinition) {
      // Use the stored parameters from the brace macro
      this.state.currentLineParams = this.state.braceMacroParams;
    } else if (this.state.continuedMacroDefinition) {
      // Old backslash-based continuation
      this.state.inMacroDefinition = true;
      // Don't reset currentLineParams - keep them from the original definition
    } else {
      // Reset state at the start of each line
      this.state.expectMacroName = false;

      // Check if this line contains a macro definition and extract parameters
      if (text.includes('#define')) {
        this.state.inMacroDefinition = true;
        this.state.currentLineParams = new Set<string>();

        // Extract parameters from the definition
        const defineMatch = text.match(/#define\s+\w+\s*\(([^)]*)\)/);
        if (defineMatch && defineMatch[1]) {
          const params = defineMatch[1].split(',').map((p) => p.trim());
          params.forEach((p) => {
            if (p) this.state.currentLineParams!.add(p);
          });
        }
      } else {
        this.state.inMacroDefinition = false;
        this.state.currentLineParams = undefined;
      }
    }

    // Reset continuation flag for this line
    this.state.continuedMacroDefinition = false;

    while (position < text.length) {
      // Check for errors at this position
      const error = this.findError(lineIndex, position, position + 1);

      // Handle multi-line comment continuation
      if (this.state.inMultiLineComment) {
        const endIndex = text.indexOf('*/', position);
        if (endIndex !== -1) {
          // Comment ends on this line
          tokens.push({
            type: 'comment',
            value: text.slice(position, endIndex + 2),
            start: position,
            end: endIndex + 2,
          });
          position = endIndex + 2;
          this.state.inMultiLineComment = false;
          continue;
        } else {
          // Comment continues to next line
          tokens.push({
            type: 'comment',
            value: text.slice(position),
            start: position,
            end: text.length,
          });
          return tokens;
        }
      }

      let matched = false;

      // Check for macro definition (#define)
      if (!matched) {
        const defineMatch = text.slice(position).match(/^#define\b/);
        if (defineMatch) {
          tokens.push({
            type: 'macro_definition', // Always treat #define as a definition
            value: defineMatch[0],
            start: position,
            end: position + defineMatch[0].length,
            error: error,
          });
          position += defineMatch[0].length;
          this.state.expectMacroName = true;
          matched = true;
        }
      }

      // Check if we're expecting a macro name after #define
      if (
        !matched &&
        this.state.expectMacroName &&
        text.slice(position).match(/^\s+/)
      ) {
        // Skip whitespace but keep expectMacroName flag
        const wsMatch = text.slice(position).match(/^\s+/)!;
        tokens.push({
          type: 'whitespace',
          value: wsMatch[0],
          start: position,
          end: position + wsMatch[0].length,
        });
        position += wsMatch[0].length;
        matched = true;
      }

      // Check for macro name after #define
      if (!matched && this.state.expectMacroName) {
        const nameMatch = text.slice(position).match(/^[a-zA-Z_]\w*/);
        if (nameMatch) {
          tokens.push({
            type: 'macro_name',
            value: nameMatch[0],
            start: position,
            end: position + nameMatch[0].length,
            error: error,
          });
          position += nameMatch[0].length;
          this.state.expectMacroName = false;

          matched = true;
        } else {
          // If we don't find a valid macro name, stop expecting one
          this.state.expectMacroName = false;
        }
      }

      // Check for macro invocation (@macroName)
      if (!matched) {
        const macroMatch = text.slice(position).match(/^@[a-zA-Z_]\w*/);
        if (macroMatch) {
          tokens.push({
            type: 'macro_invocation', // Always treat @name as invocation
            value: macroMatch[0],
            start: position,
            end: position + macroMatch[0].length,
            error: error,
          });
          position += macroMatch[0].length;
          matched = true;
        }
      }

      // Check for built-in function ({repeat, {if, {for, {reverse)
      if (!matched) {
        const builtinMatch = text
          .slice(position)
          .match(/^\{(repeat|if|for|reverse)\b/);
        if (builtinMatch) {
          // Increment brace depth for the opening brace
          this.state.braceDepth++;

          tokens.push({
            type: 'builtin_function', // Always treat these as builtin functions
            value: builtinMatch[0],
            start: position,
            end: position + builtinMatch[0].length,
            error: error,
          });
          position += builtinMatch[0].length;

          // Special handling for {for to extract loop variable
          if (builtinMatch[1] === 'for') {
            // Look ahead for the pattern: (variable in
            const forPattern = text
              .slice(position)
              .match(/^\s*\(\s*([a-zA-Z_]\w*)\s+in\b/);
            if (forPattern) {
              const loopVar = forPattern[1];
              this.state.forLoopScopes.push({
                variable: loopVar,
                depth: this.state.braceDepth,
              });
              this.state.forLoopVariables.add(loopVar);
            }
          }

          matched = true;
        }
      }

      // Check for multi-line comment start
      if (!matched && text.slice(position, position + 2) === '/*') {
        const endIndex = text.indexOf('*/', position + 2);
        if (endIndex !== -1) {
          // Single-line block comment
          tokens.push({
            type: 'comment',
            value: text.slice(position, endIndex + 2),
            start: position,
            end: endIndex + 2,
          });
          position = endIndex + 2;
        } else {
          // Multi-line comment starts
          tokens.push({
            type: 'comment',
            value: text.slice(position),
            start: position,
            end: text.length,
          });
          this.state.inMultiLineComment = true;
          return tokens;
        }
        matched = true;
      }

      // Single-line comment
      if (!matched && text.slice(position, position + 2) === '//') {
        const commentText = text.slice(position);
        const isTodoComment = /^\/\/\s*TODO:/i.test(commentText);
        const isMarkComment = /^\/\/\s*MARK:/i.test(commentText);
        tokens.push({
          type: isMarkComment
            ? 'mark_comment'
            : isTodoComment
              ? 'todo_comment'
              : 'comment',
          value: commentText,
          start: position,
          end: text.length,
        });
        return tokens;
      }

      // Standard Brainfuck operators
      // Increment/Decrement operators
      if (!matched) {
        const operatorMatch = text.slice(position).match(/^[+\-]+/);
        if (operatorMatch) {
          tokens.push({
            type: 'incdec',
            value: operatorMatch[0],
            start: position,
            end: position + operatorMatch[0].length,
            error: error,
          });
          position += operatorMatch[0].length;
          matched = true;
        }
      }

      // Brackets
      if (!matched) {
        const punctMatch = text.slice(position).match(/^[\[\]]/);
        if (punctMatch) {
          tokens.push({
            type: 'brackets',
            value: punctMatch[0],
            start: position,
            end: position + punctMatch[0].length,
            error: error,
          });
          position += punctMatch[0].length;
          matched = true;
        }
      }

      // Dot operator
      if (!matched && text[position] === '.') {
        tokens.push({
          type: 'dot',
          value: '.',
          start: position,
          end: position + 1,
          error: error,
        });
        position++;
        matched = true;
      }

      // Comma
      if (!matched && text[position] === ',') {
        tokens.push({
          type: 'comma',
          value: ',',
          start: position,
          end: position + 1,
          error: error,
        });
        position++;
        matched = true;
      }

      // Move operators <>
      if (!matched) {
        const moveMatch = text.slice(position).match(/^[<>]+/);
        if (moveMatch) {
          tokens.push({
            type: 'move',
            value: moveMatch[0],
            start: position,
            end: position + moveMatch[0].length,
            error: error,
          });
          position += moveMatch[0].length;
          matched = true;
        }
      }

      // Parentheses
      if (!matched && (text[position] === '(' || text[position] === ')')) {
        tokens.push({
          type: 'parentheses',
          value: text[position],
          start: position,
          end: position + 1,
          error: error,
        });
        position++;
        matched = true;
      }

      // Braces
      if (!matched && (text[position] === '{' || text[position] === '}')) {
        // Track brace depth for scope management
        // Only increment if this isn't part of a builtin function (which we already handled)
        const isBuiltinFunction =
          text[position] === '{' &&
          text.slice(position).match(/^\{(repeat|if|for|reverse)\b/);

        if (text[position] === '{' && !isBuiltinFunction) {
          this.state.braceDepth++;

          // Check if this is the opening brace of a macro definition
          if (
            this.state.inMacroDefinition &&
            !this.state.inBraceMacroDefinition
          ) {
            // This is the start of a brace-based macro definition
            this.state.inBraceMacroDefinition = true;
            this.state.macroDefinitionBraceDepth = 1;
            this.state.braceMacroParams = new Set(this.state.currentLineParams);
          } else if (this.state.inBraceMacroDefinition) {
            // Track nested braces within macro definition
            this.state.macroDefinitionBraceDepth!++;
          }
        } else if (text[position] === '}') {
          this.state.braceDepth--;

          // Check if we're closing a macro definition
          if (
            this.state.inBraceMacroDefinition &&
            this.state.macroDefinitionBraceDepth
          ) {
            this.state.macroDefinitionBraceDepth--;
            if (this.state.macroDefinitionBraceDepth === 0) {
              // End of macro definition
              this.state.inBraceMacroDefinition = false;
              this.state.braceMacroParams = undefined;
              this.state.currentLineParams = undefined;
            }
          }

          // Clean up for loop variables that are out of scope
          const scopesToRemove = [];
          for (let i = this.state.forLoopScopes.length - 1; i >= 0; i--) {
            if (this.state.forLoopScopes[i].depth > this.state.braceDepth) {
              scopesToRemove.push(i);
              this.state.forLoopVariables.delete(
                this.state.forLoopScopes[i].variable,
              );
            }
          }
          // Remove scopes that are no longer active
          scopesToRemove.forEach((index) =>
            this.state.forLoopScopes.splice(index, 1),
          );
        }

        tokens.push({
          type: 'braces',
          value: text[position],
          start: position,
          end: position + 1,
          error: error,
        });
        position++;
        matched = true;
      }

      // Whitespace
      if (!matched) {
        const wsMatch = text.slice(position).match(/^\s+/);
        if (wsMatch) {
          tokens.push({
            type: 'whitespace',
            value: wsMatch[0],
            start: position,
            end: position + wsMatch[0].length,
          });
          position += wsMatch[0].length;
          matched = true;
        }
      }

      // Check for numbers
      if (!matched) {
        const numberMatch = text.slice(position).match(/^\d+/);
        if (numberMatch) {
          tokens.push({
            type: 'number',
            value: numberMatch[0],
            start: position,
            end: position + numberMatch[0].length,
            error: error,
          });
          position += numberMatch[0].length;
          matched = true;
        }
      }

      // Check for 'in' keyword (for {for} loops)
      if (!matched) {
        const inMatch = text.slice(position).match(/^\bin\b/);
        if (inMatch) {
          tokens.push({
            type: 'keyword',
            value: inMatch[0],
            start: position,
            end: position + inMatch[0].length,
            error: error,
          });
          position += inMatch[0].length;
          matched = true;
        }
      }

      // Check for parameter names in macro definition or for loop variables
      if (!matched) {
        const identMatch = text.slice(position).match(/^[a-zA-Z_]\w*/);
        if (identMatch) {
          const identifier = identMatch[0];

          // Check if it's a macro parameter or a for loop variable
          const isMacroParam =
            this.state.currentLineParams &&
            this.state.currentLineParams.has(identifier);
          const isForLoopVar = this.state.forLoopVariables.has(identifier);

          if (isMacroParam || isForLoopVar) {
            tokens.push({
              type: 'parameter',
              value: identifier,
              start: position,
              end: position + identifier.length,
              error: error,
            });
            position += identifier.length;
            matched = true;
          }
        }
      }

      // Check for line continuation backslash at end of line
      if (!matched && text[position] === '\\') {
        // Check if this is at the end of the line (possibly followed by whitespace)
        const restOfLine = text.slice(position + 1).trim();
        if (restOfLine === '' && this.state.inMacroDefinition) {
          tokens.push({
            type: 'continuation',
            value: '\\',
            start: position,
            end: position + 1,
            error: error,
          });
          position++;
          matched = true;
          // Set flag to indicate next line continues the macro definition
          this.state.continuedMacroDefinition = true;
        }
      }

      // Fallback - single character
      if (!matched) {
        tokens.push({
          type: error ? 'error' : 'unknown',
          value: text[position],
          start: position,
          end: position + 1,
          error: error,
        });
        position++;
      }
    }

    return tokens;
  }

  // Initialize the async expander if not already done
  private ensureAsyncExpander() {
    if (!this.asyncExpander) {
      this.asyncExpander = createAsyncMacroExpander();
    }
  }

  // Tokenize all lines at once to maintain state
  tokenizeAllLines(lines: string[]): MacroToken[][] {
    const newText = lines.join('\n');

    // Only reset if the text has actually changed
    if (this.fullText !== newText) {
      console.log('Text changed, resetting tokenizer');
      // Reset all state including for loop tracking
      this.reset();

      // Build full text and line offsets for macro expander
      this.fullText = newText;
      this.lineOffsets = [];
      let offset = 0;

      // Calculate offset for each line start
      for (let i = 0; i < lines.length; i++) {
        this.lineOffsets.push(offset);
        offset += lines[i].length + (i < lines.length - 1 ? 1 : 0); // +1 for newline except last line
      }

      // Schedule async macro expansion
      this.scheduleAsyncExpansion();
    } else {
      console.log('Text unchanged, skipping reset');
    }

    // Tokenize each line
    return lines.map((line, index) =>
      this.tokenizeLine(line, index, index === lines.length - 1),
    );
  }

  // Schedule async macro expansion
  private scheduleAsyncExpansion() {
    this.ensureAsyncExpander();

    const fullText = this.fullText;

    // Check if we're already processing this exact text
    if (this.lastProcessedText === fullText) {
      return; // Skip redundant processing
    }

    // Cancel any pending expansion
    if (this.lastExpandPromise) {
      // Note: We can't really cancel the promise, but we can ignore its result
    }

    this.lastProcessedText = fullText;

    // Start async expansion
    this.lastExpandPromise = this.asyncExpander!.expand(fullText)
      .then((result) => {
        // Only update if this is still the latest request
        if (this.fullText === fullText) {
          console.log('Macro expansion completed:', {
            errors: result.errors,
            tokens: result.tokens,
            macros: result.macros,
          });
          this.state.expanderTokens = result.tokens;
          this.state.expanderErrors = result.errors;
          this.state.macroDefinitions = result.macros;

          // Trigger re-render by notifying listeners
          this.notifyStateChange();
        }
      })
      .catch((error) => {
        console.error('Macro expansion error:', error);
      });
  }

  // Notify listeners that state has changed
  private notifyStateChange() {
    console.log(
      'Notifying state change, callbacks:',
      this.stateChangeCallbacks.size,
    );
    // Notify all registered callbacks
    this.stateChangeCallbacks.forEach((callback) => callback());
  }

  // Register a callback for state changes
  public onStateChange(callback: () => void): () => void {
    this.stateChangeCallbacks.add(callback);
    // Return unsubscribe function
    return () => {
      this.stateChangeCallbacks.delete(callback);
    };
  }

  // Cleanup method
  destroy() {
    if (this.asyncExpander) {
      this.asyncExpander.destroy();
      this.asyncExpander = null;
    }
    this.stateChangeCallbacks.clear();
  }
}

// Token styles for macro syntax
export const enhancedMacroTokenStyles: Record<MacroToken['type'], string> = {
  macro: 'text-red-300/85', // Generic macro syntax
  macro_definition: 'text-emerald-500/90', // Verified macro definitions
  macro_name: 'text-rose-400/85', // Macro name in definition
  macro_invocation: 'text-violet-300/85 italic', // Verified macro invocations
  builtin_function: 'text-sky-400/85', // Built-in functions like repeat, if, for, reverse
  parameter: 'text-pink-300/80 italic', // Parameter references
  number: 'text-amber-400/80', // Numeric literals
  continuation: 'text-yellow-400', // Line continuation backslash
  keyword: 'text-cyan-400/85', // Keywords like 'in'
  comment: 'text-gray-400/85 italic',
  todo_comment: 'text-emerald-300/75 italic', // Beautiful dim green for TODO comments
  mark_comment: 'text-yellow-300 bg-yellow-900/30', // MARK comments with yellow background
  incdec: 'text-blue-300/85',
  brackets: 'text-orange-300/85',
  dot: 'text-teal-300/90 bg-zinc-800/40',
  comma: 'text-teal-300/80',
  move: 'text-yellow-400/80',
  parentheses: 'text-violet-200/70', // Parentheses for macro calls
  braces: 'text-sky-200/70', // Braces for builtin functions
  unknown: 'text-gray-500/75 italic',
  error: 'text-red-300/90 underline decoration-wavy', // Error styling
  whitespace: '',
};

// Export function to get errors for a specific line
export function getLineErrors(
  tokenizer: EnhancedMacroTokenizer,
  lineIndex: number,
): MacroExpansionError[] {
  if (!tokenizer.state) return [];
  return tokenizer.state.expanderErrors.filter(
    (error) => error.location && error.location.line === lineIndex,
  );
}
