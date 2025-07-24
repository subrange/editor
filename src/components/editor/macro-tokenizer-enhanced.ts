import { type ITokenizer } from "../../services/editor-manager.service";
import { createMacroExpander, type MacroToken as ExpanderToken, type MacroExpansionError } from "../../services/macro-expander";

// Token types for macro syntax
interface MacroToken {
    type: 'macro' | 'macro_definition' | 'macro_invocation' | 'builtin_function' | 
          'parameter' | 'incdec' | 'brackets' | 'move' | 'dot' | 'comma' | 
          'whitespace' | 'comment' | 'unknown' | 'error';
    value: string;
    start: number;
    end: number;
    error?: MacroExpansionError;
}

interface MacroTokenizerState {
    inMultiLineComment: boolean;
    expanderTokens: ExpanderToken[];
    expanderErrors: MacroExpansionError[];
}

export class EnhancedMacroTokenizer implements ITokenizer {
    public state: MacroTokenizerState = {
        inMultiLineComment: false,
        expanderTokens: [],
        expanderErrors: []
    };
    
    private expander = createMacroExpander();
    private fullText = '';
    private lineOffsets: number[] = [];

    reset() {
        this.state = {
            inMultiLineComment: false,
            expanderTokens: [],
            expanderErrors: []
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

    // Check if a position on a line overlaps with an expander token
    private findExpanderToken(lineIndex: number, start: number, end: number): ExpanderToken | undefined {
        const globalStart = this.lineOffsets[lineIndex] + start;
        const globalEnd = this.lineOffsets[lineIndex] + end;
        
        return this.state.expanderTokens.find(token => 
            token.range.start < globalEnd && token.range.end > globalStart
        );
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
                const defineMatch = text.slice(position).match(/^#define\s+\w+(?:\([^)]*\))?\s*.*/);
                if (defineMatch) {
                    const expanderToken = this.findExpanderToken(lineIndex, position, position + defineMatch[0].length);
                    tokens.push({
                        type: expanderToken ? 'macro_definition' : 'macro',
                        value: defineMatch[0],
                        start: position,
                        end: position + defineMatch[0].length,
                        error: error
                    });
                    position += defineMatch[0].length;
                    matched = true;
                }
            }

            // Check for macro invocation (@macroName)
            if (!matched) {
                const macroMatch = text.slice(position).match(/^@[a-zA-Z_]\w*(?:\([^)]*\))?/);
                if (macroMatch) {
                    const expanderToken = this.findExpanderToken(lineIndex, position, position + macroMatch[0].length);
                    tokens.push({
                        type: expanderToken ? 'macro_invocation' : (error ? 'error' : 'macro'),
                        value: macroMatch[0],
                        start: position,
                        end: position + macroMatch[0].length,
                        error: error
                    });
                    position += macroMatch[0].length;
                    matched = true;
                }
            }

            // Check for built-in function ({repeat(...)})
            if (!matched) {
                const builtinMatch = text.slice(position).match(/^\{repeat\s*\([^)]+\)\}/);
                if (builtinMatch) {
                    const expanderToken = this.findExpanderToken(lineIndex, position, position + builtinMatch[0].length);
                    tokens.push({
                        type: expanderToken ? 'builtin_function' : 'macro',
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
                tokens.push({
                    type: 'comment',
                    value: text.slice(position),
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
        this.lineOffsets = [0];
        let offset = 0;
        for (const line of lines) {
            offset += line.length + 1; // +1 for newline
            this.lineOffsets.push(offset);
        }
        
        // Run macro expander to get tokens and errors
        const result = this.expander.expand(this.fullText);
        this.state.expanderTokens = result.tokens;
        this.state.expanderErrors = result.errors;
        
        // Tokenize each line
        return lines.map((line, index) =>
            this.tokenizeLine(line, index, index === lines.length - 1)
        );
    }
}

// Token styles for macro syntax
export const enhancedMacroTokenStyles: Record<MacroToken['type'], string> = {
    macro: 'text-purple-400',                        // Generic macro syntax
    macro_definition: 'text-purple-500 font-bold',   // Verified macro definitions
    macro_invocation: 'text-purple-400 italic',      // Verified macro invocations
    builtin_function: 'text-cyan-400 font-bold',     // Built-in functions like repeat
    parameter: 'text-pink-400 italic',               // Parameter references
    comment: 'text-gray-500 italic',
    incdec: 'text-blue-400',
    brackets: 'text-orange-400',
    dot: 'text-teal-400 bg-zinc-700',
    comma: 'text-teal-500 bg-zinc-700',
    move: 'text-yellow-400',
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