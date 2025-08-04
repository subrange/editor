export interface Token {
    type: 'incdec' | 'brackets' | 'move' | 'dot' | 'comma' | 'whitespace' | 'comment' | 'unknown'
    value: string;
    start: number;
    end: number;
}

interface TokenizerState {
    inMultiLineComment: boolean;
}

export class FastTokenizer {
    private state: TokenizerState = {
        inMultiLineComment: false
    };

    reset() {
        this.state = {
            inMultiLineComment: false
        };
    }

    tokenizeLine(text: string, lineIndex: number, isLastLine: boolean = false): Token[] {
        const tokens: Token[] = [];
        let i = 0;
        
        // Handle multi-line comment continuation
        if (this.state.inMultiLineComment) {
            const endIndex = text.indexOf('*/');
            if (endIndex !== -1) {
                tokens.push({
                    type: 'comment',
                    value: text.substring(0, endIndex + 2),
                    start: 0,
                    end: endIndex + 2
                });
                this.state.inMultiLineComment = false;
                i = endIndex + 2;
            } else {
                tokens.push({
                    type: 'comment',
                    value: text,
                    start: 0,
                    end: text.length
                });
                return tokens;
            }
        }

        while (i < text.length) {
            const char = text[i];
            const nextChar = text[i + 1];

            // Check for comments
            if (char === '/' && nextChar === '/') {
                // Single line comment - rest of line
                tokens.push({
                    type: 'comment',
                    value: text.substring(i),
                    start: i,
                    end: text.length
                });
                break;
            } else if (char === '/' && nextChar === '*') {
                // Block comment
                const endIndex = text.indexOf('*/', i + 2);
                if (endIndex !== -1) {
                    tokens.push({
                        type: 'comment',
                        value: text.substring(i, endIndex + 2),
                        start: i,
                        end: endIndex + 2
                    });
                    i = endIndex + 2;
                } else {
                    // Multi-line comment
                    tokens.push({
                        type: 'comment',
                        value: text.substring(i),
                        start: i,
                        end: text.length
                    });
                    this.state.inMultiLineComment = true;
                    break;
                }
                continue;
            }

            // Check for => operator
            if (char === '=' && nextChar === '>') {
                tokens.push({
                    type: 'incdec',
                    value: '=>',
                    start: i,
                    end: i + 2
                });
                i += 2;
                continue;
            }

            // Single character tokens
            let type: Token['type'];
            let advance = 1;
            
            switch (char) {
                case '+':
                case '-':
                    // Collect consecutive +/- for efficiency
                    let j = i;
                    while (j < text.length && (text[j] === '+' || text[j] === '-')) {
                        j++;
                    }
                    tokens.push({
                        type: 'incdec',
                        value: text.substring(i, j),
                        start: i,
                        end: j
                    });
                    i = j;
                    continue;
                    
                case '[':
                case ']':
                    type = 'brackets';
                    break;
                    
                case '<':
                case '>':
                    type = 'move';
                    break;
                    
                case '.':
                    type = 'dot';
                    break;
                    
                case ',':
                    // Handle optional whitespace around comma
                    let start = i;
                    let end = i + 1;
                    // Skip preceding whitespace already tokenized
                    while (end < text.length && /\s/.test(text[end])) {
                        end++;
                    }
                    tokens.push({
                        type: 'comma',
                        value: text.substring(start, end),
                        start,
                        end
                    });
                    i = end;
                    continue;
                    
                case ' ':
                case '\t':
                case '\r':
                case '\n':
                    // Collect consecutive whitespace
                    let k = i;
                    while (k < text.length && /\s/.test(text[k])) {
                        k++;
                    }
                    tokens.push({
                        type: 'whitespace',
                        value: text.substring(i, k),
                        start: i,
                        end: k
                    });
                    i = k;
                    continue;
                    
                default:
                    type = 'unknown';
            }

            tokens.push({
                type,
                value: char,
                start: i,
                end: i + advance
            });
            i += advance;
        }

        return tokens;
    }

    tokenizeAllLines(lines: string[]): Token[][] {
        this.reset();
        return lines.map((line, index) =>
            this.tokenizeLine(line, index, index === lines.length - 1)
        );
    }
}

export const tokenStyles: Record<Token['type'], string> = {
    comment: 'text-gray-500 italic',
    incdec: 'text-blue-400/80',
    brackets: 'text-orange-400/80',
    dot: 'text-teal-400 bg-zinc-700',
    comma: 'text-teal-500 bg-zinc-700',
    move: 'text-yellow-400/80',
    unknown: 'text-gray-500 italic',
    whitespace: ''
};