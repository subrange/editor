export interface Token {
    type: 'regular' | 'incdec' | 'brackets' | 'move' | 'dot' | 'comma' | 'whitespace' | 'comment' | 'unknown'
    value: string;
    start: number;
    end: number;
}

export class DummyTokenizer {
    reset() {
    }

    tokenizeLine(text: string, lineIndex: number, isLastLine: boolean = false): Token[] {
        const tokens: Token[] = [];

        tokens.push({
            type: 'regular',
            value: text,
            start: 0,
            end: text.length
        })

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
    whitespace: '',
    regular: 'text-gray-300'
};