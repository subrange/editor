interface Token {
  type:
    | 'incdec'
    | 'brackets'
    | 'move'
    | 'dot'
    | 'comma'
    | 'whitespace'
    | 'comment'
    | 'unknown';
  value: string;
  start: number;
  end: number;
}

interface TokenizerState {
  inMultiLineComment: boolean;
}

// Pre-compiled regex for all token types
const TOKEN_REGEX =
  /\/\*[\s\S]*?\*\/|\/\*[\s\S]*$|\/\/.*$|[+\-]+|=>|[\[\]]|\.|\s*,\s*|[<>]|\s+|./g;

export class OptimizedBFTokenizer {
  private state: TokenizerState = {
    inMultiLineComment: false,
  };

  reset() {
    this.state = {
      inMultiLineComment: false,
    };
  }

  tokenizeLine(
    text: string,
    lineIndex: number,
    isLastLine: boolean = false,
  ): Token[] {
    const tokens: Token[] = [];

    // Handle multi-line comment continuation
    if (this.state.inMultiLineComment) {
      const endIndex = text.indexOf('*/');
      if (endIndex !== -1) {
        tokens.push({
          type: 'comment',
          value: text.slice(0, endIndex + 2),
          start: 0,
          end: endIndex + 2,
        });
        this.state.inMultiLineComment = false;
        // Process rest of line
        const restTokens = this.tokenizeLine(
          text.slice(endIndex + 2),
          lineIndex,
          isLastLine,
        );
        return tokens.concat(
          restTokens.map((t) => ({
            ...t,
            start: t.start + endIndex + 2,
            end: t.end + endIndex + 2,
          })),
        );
      } else {
        tokens.push({
          type: 'comment',
          value: text,
          start: 0,
          end: text.length,
        });
        return tokens;
      }
    }

    // Single regex pass for all tokens
    TOKEN_REGEX.lastIndex = 0;
    let match;

    while ((match = TOKEN_REGEX.exec(text)) !== null) {
      const value = match[0];
      const start = match.index;
      const end = start + value.length;

      let type: Token['type'];

      // Determine token type
      if (value.startsWith('/*')) {
        type = 'comment';
        if (!value.endsWith('*/')) {
          this.state.inMultiLineComment = true;
        }
      } else if (value.startsWith('//')) {
        type = 'comment';
      } else if (/^[+\-]+$/.test(value) || value === '=>') {
        type = 'incdec';
      } else if (value === '[' || value === ']') {
        type = 'brackets';
      } else if (value === '.') {
        type = 'dot';
      } else if (value.includes(',')) {
        type = 'comma';
      } else if (value === '<' || value === '>') {
        type = 'move';
      } else if (/^\s+$/.test(value)) {
        type = 'whitespace';
      } else {
        type = 'unknown';
      }

      tokens.push({ type, value, start, end });
    }

    return tokens;
  }

  tokenizeAllLines(lines: string[]): Token[][] {
    this.reset();
    return lines.map((line, index) =>
      this.tokenizeLine(line, index, index === lines.length - 1),
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
};
