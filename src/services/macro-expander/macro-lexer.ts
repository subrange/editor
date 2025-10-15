export const TokenType = {
  // Literals and identifiers
  IDENTIFIER: 'IDENTIFIER',
  NUMBER: 'NUMBER',

  // Macro-specific tokens
  DEFINE: 'DEFINE', // #define
  AT: 'AT', // @
  HASH: 'HASH', // # (for #macro invocations)

  // Brainfuck commands
  BF_COMMAND: 'BF_COMMAND', // ><+-.,[]$

  // Builtin functions
  BUILTIN_REPEAT: 'BUILTIN_REPEAT', // {repeat
  BUILTIN_IF: 'BUILTIN_IF', // {if
  BUILTIN_FOR: 'BUILTIN_FOR', // {for
  BUILTIN_REVERSE: 'BUILTIN_REVERSE', // {reverse
  BUILTIN_PRESERVE: 'BUILTIN_PRESERVE', // {preserve
  COLON_SHORTHAND: 'COLON_SHORTHAND', // {: (shorthand for preserve)

  // Delimiters
  LPAREN: 'LPAREN', // (
  RPAREN: 'RPAREN', // )
  LBRACE: 'LBRACE', // {
  RBRACE: 'RBRACE', // }
  COMMA: 'COMMA', // ,
  IN: 'IN', // in keyword for for loops

  // Line continuation
  BACKSLASH: 'BACKSLASH', // \ at end of line

  // Comments
  COMMENT_SINGLE: 'COMMENT_SINGLE', // //
  COMMENT_MULTI: 'COMMENT_MULTI', // /* */

  // Whitespace and newlines
  WHITESPACE: 'WHITESPACE',
  NEWLINE: 'NEWLINE',

  // End of file
  EOF: 'EOF',

  // Unknown/Other
  OTHER: 'OTHER',
} as const;

export type TokenType = (typeof TokenType)[keyof typeof TokenType];

export interface Token {
  type: TokenType;
  value: string;
  position: {
    start: number;
    end: number;
    line: number;
    column: number;
  };
}

export interface LexerOptions {
  preserveWhitespace?: boolean;
  preserveComments?: boolean;
}

export class MacroLexer {
  private input: string;
  private position: number = 0;
  private line: number = 1;
  private column: number = 1;
  private tokens: Token[] = [];
  private options: LexerOptions;

  constructor(input: string, options: LexerOptions = {}) {
    this.input = input;
    this.options = {
      preserveWhitespace: false,
      preserveComments: false,
      ...options,
    };
  }

  tokenize(): Token[] {
    this.tokens = [];
    this.position = 0;
    this.line = 1;
    this.column = 1;

    while (this.position < this.input.length) {
      this.skipWhitespaceIfNeeded();

      if (this.position >= this.input.length) break;

      const token = this.nextToken();
      if (token) {
        if (this.shouldIncludeToken(token)) {
          this.tokens.push(token);
        }
      }
    }

    // Add EOF token
    this.tokens.push(
      this.createToken(TokenType.EOF, '', this.position, this.position),
    );
    return this.tokens;
  }

  private shouldIncludeToken(token: Token): boolean {
    if (
      !this.options.preserveWhitespace &&
      token.type === TokenType.WHITESPACE
    ) {
      return false;
    }
    if (
      !this.options.preserveComments &&
      (token.type === TokenType.COMMENT_SINGLE ||
        token.type === TokenType.COMMENT_MULTI)
    ) {
      return false;
    }
    return true;
  }

  private nextToken(): Token | null {
    const start = this.position;

    // Check for line continuation (backslash at end of line)
    if (this.peek() === '\\' && this.peekAhead(1) === '\n') {
      this.advance(); // consume \
      this.advance(); // consume \n
      return this.createToken(TokenType.BACKSLASH, '\\', start, this.position);
    }

    // Check for newline
    if (this.peek() === '\n') {
      this.advance();
      return this.createToken(TokenType.NEWLINE, '\n', start, this.position);
    }

    // Check for #define
    if (this.match('#define')) {
      return this.createToken(
        TokenType.DEFINE,
        '#define',
        start,
        this.position,
      );
    }

    // Check for single-line comment
    if (this.match('//')) {
      const value = this.consumeUntil('\n');
      return this.createToken(
        TokenType.COMMENT_SINGLE,
        '//' + value,
        start,
        this.position,
      );
    }

    // Check for multi-line comment
    if (this.match('/*')) {
      const value = this.consumeMultilineComment();
      return this.createToken(
        TokenType.COMMENT_MULTI,
        '/*' + value + '*/',
        start,
        this.position,
      );
    }

    // Check for builtin functions and shorthand
    // Check {: shorthand first
    if (this.match('{:')) {
      return this.createToken(
        TokenType.COLON_SHORTHAND,
        '{:',
        start,
        this.position,
      );
    }

    if (this.match('{repeat')) {
      return this.createToken(
        TokenType.BUILTIN_REPEAT,
        '{repeat',
        start,
        this.position,
      );
    }

    if (this.match('{if')) {
      return this.createToken(
        TokenType.BUILTIN_IF,
        '{if',
        start,
        this.position,
      );
    }

    if (this.match('{for')) {
      return this.createToken(
        TokenType.BUILTIN_FOR,
        '{for',
        start,
        this.position,
      );
    }

    if (this.match('{reverse')) {
      return this.createToken(
        TokenType.BUILTIN_REVERSE,
        '{reverse',
        start,
        this.position,
      );
    }

    if (this.match('{preserve')) {
      return this.createToken(
        TokenType.BUILTIN_PRESERVE,
        '{preserve',
        start,
        this.position,
      );
    }

    // Check for @ symbol
    if (this.peek() === '@') {
      this.advance();
      return this.createToken(TokenType.AT, '@', start, this.position);
    }

    // Check for # symbol (not part of #define which was already handled)
    if (this.peek() === '#') {
      this.advance();
      return this.createToken(TokenType.HASH, '#', start, this.position);
    }

    // Check for identifiers (must come after @ and # check)
    if (this.isAlpha(this.peek())) {
      const value = this.consumeWhile((ch) => this.isAlphaNumeric(ch));
      // Check for 'in' keyword
      if (value === 'in') {
        return this.createToken(TokenType.IN, value, start, this.position);
      }
      return this.createToken(
        TokenType.IDENTIFIER,
        value,
        start,
        this.position,
      );
    }

    // Check for character literals 'c'
    if (this.peek() === "'") {
      // Look ahead to check if it's a valid character literal
      let checkPos = 1;
      let escaped = false;

      if (this.peekAhead(checkPos) === '\\') {
        // Escape sequence
        escaped = true;
        checkPos = 2;
      }

      // Check if there's a closing quote at the right position
      const expectedClosePos = escaped ? 3 : 2;
      if (this.peekAhead(expectedClosePos) === "'") {
        this.advance(); // consume opening '

        let charValue: string;
        if (escaped) {
          this.advance(); // consume \
          const escapeChar = this.advance();
          // Handle common escape sequences
          switch (escapeChar) {
            case 'n':
              charValue = '\n';
              break;
            case 't':
              charValue = '\t';
              break;
            case 'r':
              charValue = '\r';
              break;
            case '\\':
              charValue = '\\';
              break;
            case "'":
              charValue = "'";
              break;
            case '0':
              charValue = '\0';
              break;
            default:
              charValue = escapeChar; // Unknown escape, use literal
          }
        } else {
          charValue = this.advance(); // consume the character
        }

        this.advance(); // consume closing '

        // Store the full literal representation for tokenizer
        let fullLiteral: string;
        if (escaped) {
          // Preserve the escape sequence in the token value
          const escapeSequence =
            charValue === '\n'
              ? 'n'
              : charValue === '\t'
                ? 't'
                : charValue === '\r'
                  ? 'r'
                  : charValue === '\\'
                    ? '\\'
                    : charValue === "'"
                      ? "'"
                      : charValue === '\0'
                        ? '0'
                        : escapeChar; // Use the original escape char for unknown sequences
          fullLiteral = `'\\${escapeSequence}'`;
        } else {
          fullLiteral = `'${charValue}'`;
        }
        return this.createToken(
          TokenType.NUMBER,
          fullLiteral,
          start,
          this.position,
        );
      }
    }

    // Check for numbers (including hexadecimal)
    if (
      this.isDigit(this.peek()) ||
      (this.peek() === '-' && this.isDigit(this.peekAhead(1))) ||
      (this.peek() === '0' &&
        (this.peekAhead(1) === 'x' || this.peekAhead(1) === 'X'))
    ) {
      const value = this.consumeNumber();
      return this.createToken(TokenType.NUMBER, value, start, this.position);
    }

    // Check for delimiters first (including comma)
    const ch = this.peek();
    switch (ch) {
      case '(':
        this.advance();
        return this.createToken(TokenType.LPAREN, '(', start, this.position);
      case ')':
        this.advance();
        return this.createToken(TokenType.RPAREN, ')', start, this.position);
      case '{':
        this.advance();
        return this.createToken(TokenType.LBRACE, '{', start, this.position);
      case '}':
        this.advance();
        return this.createToken(TokenType.RBRACE, '}', start, this.position);
      case ',':
        this.advance();
        return this.createToken(TokenType.COMMA, ',', start, this.position);
    }

    // Check for Brainfuck commands (now comma will be treated as delimiter above)
    if (this.isBfCommand(this.peek())) {
      const value = this.consumeWhile((ch) => this.isBfCommand(ch));
      return this.createToken(
        TokenType.BF_COMMAND,
        value,
        start,
        this.position,
      );
    }

    // Check for whitespace
    if (this.isWhitespace(this.peek())) {
      const value = this.consumeWhile(
        (ch) => this.isWhitespace(ch) && ch !== '\n',
      );
      return this.createToken(
        TokenType.WHITESPACE,
        value,
        start,
        this.position,
      );
    }

    // Unknown character
    const value = this.advance();
    return this.createToken(TokenType.OTHER, value, start, this.position);
  }

  private createToken(
    type: TokenType,
    value: string,
    start: number,
    end: number,
  ): Token {
    // Calculate the line and column for the start position
    let line = 1;
    let column = 1;
    for (let i = 0; i < start; i++) {
      if (this.input[i] === '\n') {
        line++;
        column = 1;
      } else {
        column++;
      }
    }

    return {
      type,
      value,
      position: {
        start,
        end,
        line,
        column,
      },
    };
  }

  private peek(): string {
    if (this.position >= this.input.length) return '\0';
    return this.input[this.position];
  }

  private peekAhead(offset: number): string {
    const pos = this.position + offset;
    if (pos >= this.input.length) return '\0';
    return this.input[pos];
  }

  private advance(): string {
    const ch = this.input[this.position++];
    if (ch === '\n') {
      this.line++;
      this.column = 1;
    } else {
      this.column++;
    }
    return ch;
  }

  private match(str: string): boolean {
    if (this.position + str.length > this.input.length) return false;

    for (let i = 0; i < str.length; i++) {
      if (this.input[this.position + i] !== str[i]) return false;
    }

    // Advance position
    for (let i = 0; i < str.length; i++) {
      this.advance();
    }

    return true;
  }

  private consumeWhile(predicate: (ch: string) => boolean): string {
    let result = '';
    while (this.position < this.input.length && predicate(this.peek())) {
      result += this.advance();
    }
    return result;
  }

  private consumeUntil(ch: string): string {
    let result = '';
    while (this.position < this.input.length && this.peek() !== ch) {
      result += this.advance();
    }
    return result;
  }

  private consumeMultilineComment(): string {
    let result = '';
    while (this.position < this.input.length) {
      if (this.peek() === '*' && this.peekAhead(1) === '/') {
        this.advance(); // consume *
        this.advance(); // consume /
        break;
      }
      result += this.advance();
    }
    return result;
  }

  private consumeNumber(): string {
    let result = '';

    // Handle negative sign
    if (this.peek() === '-') {
      result += this.advance();
    }

    // Check for hexadecimal prefix
    if (
      this.peek() === '0' &&
      (this.peekAhead(1) === 'x' || this.peekAhead(1) === 'X')
    ) {
      result += this.advance(); // consume '0'
      result += this.advance(); // consume 'x' or 'X'
      // Consume hex digits
      result += this.consumeWhile((ch) => this.isHexDigit(ch));
    } else {
      // Consume decimal digits
      result += this.consumeWhile((ch) => this.isDigit(ch));
    }

    return result;
  }

  private skipWhitespaceIfNeeded(): void {
    if (!this.options.preserveWhitespace) {
      while (
        this.position < this.input.length &&
        this.isWhitespace(this.peek()) &&
        this.peek() !== '\n'
      ) {
        this.advance();
      }
    }
  }

  private isAlpha(ch: string): boolean {
    return /[a-zA-Z_]/.test(ch);
  }

  private isDigit(ch: string): boolean {
    return /[0-9]/.test(ch);
  }

  private isHexDigit(ch: string): boolean {
    return /[0-9a-fA-F]/.test(ch);
  }

  private isAlphaNumeric(ch: string): boolean {
    return /[a-zA-Z0-9_]/.test(ch);
  }

  private isWhitespace(ch: string): boolean {
    return /\s/.test(ch);
  }

  private isBfCommand(ch: string): boolean {
    return /[><+\-.\[\]$]/.test(ch);
  }
}

// Helper function to tokenize with default options
export function tokenize(input: string, options?: LexerOptions): Token[] {
  const lexer = new MacroLexer(input, options);
  return lexer.tokenize();
}
