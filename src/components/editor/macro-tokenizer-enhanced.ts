import { type ITokenizer } from "../../services/editor-manager.service";
import {
    createMacroExpander,
    type MacroToken as ExpanderToken,
    type MacroExpansionError,
    type MacroDefinition
} from "../../services/macro-expander/macro-expander.ts";

// Token types for macro syntax
export interface MacroToken {
    type: 'macro' | 'macro_definition' | 'macro_invocation' | 'builtin_function' | 
          'parameter' | 'incdec' | 'brackets' | 'move' | 'dot' | 'comma' | 
          'whitespace' | 'comment' | 'todo_comment' | 'mark_comment' | 'unknown' | 'error' | 'parentheses' | 'braces' | 'macro_name' | 'number' | 'continuation';
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
    inMacroDefinition?: boolean; // Track if we're in a macro definition line
    currentLineParams?: Set<string>;  // Parameters for current macro definition line
    continuedMacroDefinition?: boolean; // Track if previous line ended with backslash
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
            macroDefinitions: [],
            continuedMacroDefinition: false
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
        
        // Check if we're continuing a macro definition from previous line
        if (this.state.continuedMacroDefinition) {
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
                    const params = defineMatch[1].split(',').map(p => p.trim());
                    params.forEach(p => {
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

            // Check for built-in function ({repeat or {if)
            if (!matched) {
                const builtinMatch = text.slice(position).match(/^\{(repeat|if)\b/);
                if (builtinMatch) {
                    tokens.push({
                        type: 'builtin_function',  // Always treat {repeat and {if as builtin
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
                const isMarkComment = /^\/\/\s*MARK:/i.test(commentText);
                tokens.push({
                    type: isMarkComment ? 'mark_comment' : (isTodoComment ? 'todo_comment' : 'comment'),
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

            // Check for numbers
            if (!matched) {
                const numberMatch = text.slice(position).match(/^\d+/);
                if (numberMatch) {
                    tokens.push({
                        type: 'number',
                        value: numberMatch[0],
                        start: position,
                        end: position + numberMatch[0].length,
                        error: error
                    });
                    position += numberMatch[0].length;
                    matched = true;
                }
            }

            // Check for parameter names in macro definition
            if (!matched && this.state.currentLineParams && this.state.currentLineParams.size > 0) {
                const identMatch = text.slice(position).match(/^[a-zA-Z_]\w*/);
                if (identMatch && this.state.currentLineParams.has(identMatch[0])) {
                    tokens.push({
                        type: 'parameter',
                        value: identMatch[0],
                        start: position,
                        end: position + identMatch[0].length,
                        error: error
                    });
                    position += identMatch[0].length;
                    matched = true;
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
                        error: error
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
    macro: 'text-red-300/85',                        // Generic macro syntax
    macro_definition: 'text-emerald-500/90',         // Verified macro definitions
    macro_name: 'text-rose-400/85',                  // Macro name in definition
    macro_invocation: 'text-violet-300/85 italic',   // Verified macro invocations
    builtin_function: 'text-sky-400/85',             // Built-in functions like repeat
    parameter: 'text-pink-300/80 italic',            // Parameter references
    number: 'text-amber-400/80',                     // Numeric literals
    continuation: 'text-yellow-400',                 // Line continuation backslash
    comment: 'text-gray-400/85 italic',
    todo_comment: 'text-emerald-300/75 italic',      // Beautiful dim green for TODO comments
    mark_comment: 'text-yellow-300 bg-yellow-900/30', // MARK comments with yellow background
    incdec: 'text-blue-300/85',
    brackets: 'text-orange-300/85',
    dot: 'text-teal-300/90 bg-zinc-800/40',
    comma: 'text-teal-300/80',
    move: 'text-yellow-400/80',
    parentheses: 'text-violet-200/70',               // Parentheses for macro calls
    braces: 'text-sky-200/70',                       // Braces for builtin functions
    unknown: 'text-gray-500/75 italic',
    error: 'text-red-300/90 underline decoration-wavy', // Error styling
    whitespace: ''
};

// Export function to get errors for a specific line
export function getLineErrors(tokenizer: EnhancedMacroTokenizer, lineIndex: number): MacroExpansionError[] {
    if (!tokenizer.state) return [];
    return tokenizer.state.expanderErrors.filter(error => 
        error.location && error.location.line === lineIndex
    );
}