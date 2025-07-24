import { type ITokenizer } from "../../services/editor-manager.service";

// Token types for macro syntax
interface MacroToken {
    type: 'macro' | 'parameter' | 'incdec' | 'brackets' | 'move' | 'dot' | 'comma' | 'whitespace' | 'comment' | 'unknown';
    value: string;
    start: number;
    end: number;
}

interface MacroTokenizerState {
    inMultiLineComment: boolean;
    inMacroDefinition: boolean;
}

export class MacroTokenizer implements ITokenizer {
    private state: MacroTokenizerState = {
        inMultiLineComment: false,
        inMacroDefinition: false
    };

    reset() {
        this.state = {
            inMultiLineComment: false,
            inMacroDefinition: false
        };
    }

    tokenizeLine(text: string, lineIndex: number, isLastLine: boolean = false): MacroToken[] {
        const tokens: MacroToken[] = [];
        let position = 0;

        while (position < text.length) {
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

            // Check for macro definition (e.g., @macroName or @macro(param1, param2))
            if (!matched) {
                const macroMatch = text.slice(position).match(/^@[a-zA-Z_]\w*(\([^)]*\))?/);
                if (macroMatch) {
                    tokens.push({
                        type: 'macro',
                        value: macroMatch[0],
                        start: position,
                        end: position + macroMatch[0].length
                    });
                    position += macroMatch[0].length;
                    matched = true;
                }
            }

            // Check for parameter reference (e.g., $1, $2, $param)
            if (!matched) {
                const paramMatch = text.slice(position).match(/^\$[a-zA-Z0-9_]+/);
                if (paramMatch) {
                    tokens.push({
                        type: 'parameter',
                        value: paramMatch[0],
                        start: position,
                        end: position + paramMatch[0].length
                    });
                    position += paramMatch[0].length;
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
                const operatorMatch = text.slice(position).match(/^([+\-]+|=>)/);
                if (operatorMatch) {
                    tokens.push({
                        type: 'incdec',
                        value: operatorMatch[0],
                        start: position,
                        end: position + operatorMatch[0].length
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
                        end: position + punctMatch[0].length
                    });
                    position += punctMatch[0].length;
                    matched = true;
                }
            }

            // Dot operator
            if (!matched) {
                const dotMatch = text.slice(position).match(/^\./);
                if (dotMatch) {
                    tokens.push({
                        type: 'dot',
                        value: dotMatch[0],
                        start: position,
                        end: position + dotMatch[0].length
                    });
                    position += dotMatch[0].length;
                    matched = true;
                }
            }

            // Comma
            if (!matched) {
                const commaMatch = text.slice(position).match(/^,/);
                if (commaMatch) {
                    tokens.push({
                        type: 'comma',
                        value: commaMatch[0],
                        start: position,
                        end: position + commaMatch[0].length
                    });
                    position += commaMatch[0].length;
                    matched = true;
                }
            }

            // Move operators <>
            if (!matched) {
                const moveMatch = text.slice(position).match(/^[<>]+/);
                if (moveMatch) {
                    tokens.push({
                        type: 'move',
                        value: moveMatch[0],
                        start: position,
                        end: position + moveMatch[0].length
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
                    type: 'unknown',
                    value: text[position],
                    start: position,
                    end: position + 1
                });
                position++;
            }
        }

        return tokens;
    }

    // Tokenize all lines at once to maintain state
    tokenizeAllLines(lines: string[]): MacroToken[][] {
        this.reset();
        return lines.map((line, index) =>
            this.tokenizeLine(line, index, index === lines.length - 1)
        );
    }
}

// Token styles for macro syntax
export const macroTokenStyles: Record<MacroToken['type'], string> = {
    macro: 'text-purple-400 font-bold',      // Macro definitions/calls
    parameter: 'text-pink-400 italic',       // Parameter references
    comment: 'text-gray-500 italic',
    incdec: 'text-blue-400',
    brackets: 'text-orange-400',
    dot: 'text-teal-400 bg-zinc-700',
    comma: 'text-teal-500 bg-zinc-700',
    move: 'text-yellow-400',
    unknown: 'text-gray-500 italic',
    whitespace: ''
};