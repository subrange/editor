import { type ITokenizer } from "../../services/editor-manager.service";
import {
    createMacroExpander,
    type MacroToken as ExpanderToken,
    type MacroExpansionError,
    type MacroDefinition
} from "../../services/macro-expander";

// Token types for macro syntax
export interface MacroToken {
    type: 'macro' | 'macro_definition' | 'macro_invocation' | 'builtin_function' | 
          'parameter' | 'incdec' | 'brackets' | 'move' | 'dot' | 'comma' | 
          'whitespace' | 'comment' | 'todo_comment' | 'unknown' | 'error' | 'parentheses' | 'braces' | 'macro_name';
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
    expectMacroName?: boolean;  // Track if we just saw #define
}

export class EnhancedMacroTokenizer implements ITokenizer {
    public state: MacroTokenizerState = {
        inMultiLineComment: false,
        expanderTokens: [],
        expanderErrors: [],
        macroDefinitions: []
    };
    
    private expander = createMacroExpander();
    private fullText = '';
    private lineOffsets: number[] = [];

    reset() {
        this.state = {
            inMultiLineComment: false,
            expanderTokens: [],
            expanderErrors: [],
            macroDefinitions: []
        };
        this.fullText = '';
        this.lineOffsets = [];
    }

    // Convert global position to line/column
    private positionToLineColumn(position: number): { line: number, column: number } {
        for (let i = 0; i < this.lineOffsets.length - 1; i++) {
            if (position >= this.lineOffsets[i] && position < this.lineOffsets[i + 1]) {
                return { line: i, column: position - this.lineOffsets[i] };
            }
        }
        // Last line
        const lastLine = this.lineOffsets.length - 1;
        return { line: lastLine, column: position - this.lineOffsets[lastLine] };
    }


    // Check if a position on a line has an error
    private findError(lineIndex: number, start: number, end: number): MacroExpansionError | undefined {
        return this.state.expanderErrors.find(error => {
            if (!error.location) return false;
            return error.location.line === lineIndex &&
                   error.location.column < end &&
                   error.location.column + error.location.length > start;
        });
    }

    tokenizeLine(text: string, lineIndex: number, isLastLine: boolean = false): MacroToken[] {
        const tokens: MacroToken[] = [];
        let position = 0;
        
        // Reset expectMacroName at the start of each line (in case previous line ended incomplete)
        this.state.expectMacroName = false;

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
                        end: endIndex + 2
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
                        end: text.length
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
                        type: 'macro_definition',  // Always treat #define as a definition
                        value: defineMatch[0],
                        start: position,
                        end: position + defineMatch[0].length,
                        error: error
                    });
                    position += defineMatch[0].length;
                    this.state.expectMacroName = true;
                    matched = true;
                }
            }

            // Check if we're expecting a macro name after #define
            if (!matched && this.state.expectMacroName && text.slice(position).match(/^\s+/)) {
                // Skip whitespace but keep expectMacroName flag
                const wsMatch = text.slice(position).match(/^\s+/)!;
                tokens.push({
                    type: 'whitespace',
                    value: wsMatch[0],
                    start: position,
                    end: position + wsMatch[0].length
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
                        error: error
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
                        type: 'macro_invocation',  // Always treat @name as invocation
                        value: macroMatch[0],
                        start: position,
                        end: position + macroMatch[0].length,
                        error: error
                    });
                    position += macroMatch[0].length;
                    matched = true;
                }
            }

            // Check for built-in function ({repeat)
            if (!matched) {
                const builtinMatch = text.slice(position).match(/^\{repeat\b/);
                if (builtinMatch) {
                    tokens.push({
                        type: 'builtin_function',  // Always treat {repeat as builtin
                        value: builtinMatch[0],
                        start: position,
                        end: position + builtinMatch[0].length,
                        error: error
                    });
                    position += builtinMatch[0].length;
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
                        end: endIndex + 2
                    });
                    position = endIndex + 2;
                } else {
                    // Multi-line comment starts
                    tokens.push({
                        type: 'comment',
                        value: text.slice(position),
                        start: position,
                        end: text.length
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
                tokens.push({
                    type: isTodoComment ? 'todo_comment' : 'comment',
                    value: commentText,
                    start: position,
                    end: text.length
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
                        error: error
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
                        error: error
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
                    error: error
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
                    error: error
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
                        error: error
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
                    error: error
                });
                position++;
                matched = true;
            }

            // Braces
            if (!matched && (text[position] === '{' || text[position] === '}')) {
                tokens.push({
                    type: 'braces',
                    value: text[position],
                    start: position,
                    end: position + 1,
                    error: error
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
                        end: position + wsMatch[0].length
                    });
                    position += wsMatch[0].length;
                    matched = true;
                }
            }

            // Fallback - single character
            if (!matched) {
                tokens.push({
                    type: error ? 'error' : 'unknown',
                    value: text[position],
                    start: position,
                    end: position + 1,
                    error: error
                });
                position++;
            }
        }

        return tokens;
    }

    // Tokenize all lines at once to maintain state
    tokenizeAllLines(lines: string[]): MacroToken[][] {
        this.reset();
        
        // Build full text and line offsets for macro expander
        this.fullText = lines.join('\n');
        this.lineOffsets = [];
        let offset = 0;
        
        // Calculate offset for each line start
        for (let i = 0; i < lines.length; i++) {
            this.lineOffsets.push(offset);
            offset += lines[i].length + (i < lines.length - 1 ? 1 : 0); // +1 for newline except last line
        }
        
        // Run macro expander to get tokens, errors, and macro definitions
        const result = this.expander.expand(this.fullText);
        this.state.expanderTokens = result.tokens;
        this.state.expanderErrors = result.errors;
        this.state.macroDefinitions = result.macros;
        
        // Tokenize each line
        return lines.map((line, index) =>
            this.tokenizeLine(line, index, index === lines.length - 1)
        );
    }
}

// Token styles for macro syntax
export const enhancedMacroTokenStyles: Record<MacroToken['type'], string> = {
    macro: 'text-red-400',                        // Generic macro syntax
    macro_definition: 'text-green-600',   // Verified macro definitions
    macro_name: 'text-pink-600',         // Macro name in definition
    macro_invocation: 'text-purple-400 italic',      // Verified macro invocations
    builtin_function: 'text-cyan-400',     // Built-in functions like repeat
    parameter: 'text-pink-400 italic',               // Parameter references
    comment: 'text-gray-500 italic',
    todo_comment: 'text-green-400/70 italic',        // Beautiful dim green for TODO comments
    incdec: 'text-blue-400',
    brackets: 'text-orange-400',
    dot: 'text-teal-400 bg-zinc-700',
    comma: 'text-teal-500',
    move: 'text-yellow-400',
    parentheses: 'text-purple-300',                  // Parentheses for macro calls
    braces: 'text-cyan-300',                         // Braces for builtin functions
    unknown: 'text-gray-500 italic',
    error: 'text-red-500 underline decoration-wavy', // Error styling
    whitespace: ''
};

// Export function to get errors for a specific line
export function getLineErrors(tokenizer: EnhancedMacroTokenizer, lineIndex: number): MacroExpansionError[] {
    if (!tokenizer.state) return [];
    return tokenizer.state.expanderErrors.filter(error => 
        error.location && error.location.line === lineIndex
    );
}