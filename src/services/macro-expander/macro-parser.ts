import { tokenize, TokenType } from './macro-lexer.ts';
import type { Token, LexerOptions } from './macro-lexer.ts';
import type { MacroExpansionError, MacroToken } from './macro-expander.ts';

export interface ASTNode {
  type: string;
  position: {
    start: number;
    end: number;
    line: number;
    column: number;
  };
}

export interface ProgramNode extends ASTNode {
  type: 'Program';
  statements: StatementNode[];
}

export type StatementNode = MacroDefinitionNode | CodeLineNode;

export interface MacroDefinitionNode extends ASTNode {
  type: 'MacroDefinition';
  name: string;
  parameters?: string[];
  body: BodyNode[];
}

export interface CodeLineNode extends ASTNode {
  type: 'CodeLine';
  content: ContentNode[];
}

export type ContentNode =
  | BrainfuckCommandNode
  | MacroInvocationNode
  | BuiltinFunctionNode
  | TextNode;

export interface BrainfuckCommandNode extends ASTNode {
  type: 'BrainfuckCommand';
  commands: string;
}

export interface MacroInvocationNode extends ASTNode {
  type: 'MacroInvocation';
  name: string;
  arguments?: ExpressionNode[];
}

export interface BuiltinFunctionNode extends ASTNode {
  type: 'BuiltinFunction';
  name: 'repeat' | 'if' | 'for' | 'reverse' | 'preserve';
  arguments: ExpressionNode[];
}

export interface ArrayLiteralNode extends ASTNode {
  type: 'ArrayLiteral';
  elements: ExpressionNode[];
}

export type ExpressionNode =
  | NumberNode
  | IdentifierNode
  | MacroInvocationNode
  | BuiltinFunctionNode
  | ExpressionListNode
  | TextNode
  | BrainfuckCommandNode
  | ArrayLiteralNode
  | TuplePatternNode;

export interface NumberNode extends ASTNode {
  type: 'Number';
  value: number;
}

export interface IdentifierNode extends ASTNode {
  type: 'Identifier';
  name: string;
}

export interface TuplePatternNode extends ASTNode {
  type: 'TuplePattern';
  elements: string[];
}

export interface ExpressionListNode extends ASTNode {
  type: 'ExpressionList';
  expressions: ContentNode[];
}

export interface TextNode extends ASTNode {
  type: 'Text';
  value: string;
}

export type BodyNode = ContentNode;

export interface ParseResult {
  ast: ProgramNode;
  errors: MacroExpansionError[];
  tokens: MacroToken[];
}

export class MacroParser {
  private tokens: Token[];
  private current: number = 0;
  private errors: MacroExpansionError[] = [];
  private macroTokens: MacroToken[] = [];

  constructor(tokens: Token[]) {
    this.tokens = tokens;
  }

  parse(): ParseResult {
    const statements: StatementNode[] = [];

    while (!this.isAtEnd()) {
      // Skip only newlines at statement level, preserve whitespace for code lines
      while (this.match(TokenType.NEWLINE)) {
        // Continue
      }

      if (this.isAtEnd()) break;

      const stmt = this.parseStatement();
      if (stmt) {
        statements.push(stmt);
      }
    }

    const ast: ProgramNode = {
      type: 'Program',
      statements,
      position: {
        start: 0,
        end: this.tokens[this.tokens.length - 1]?.position.end || 0,
        line: 1,
        column: 1,
      },
    };

    return {
      ast,
      errors: this.errors,
      tokens: this.macroTokens,
    };
  }

  private parseStatement(): StatementNode | null {
    // Save current position to potentially backtrack
    const savedPosition = this.current;

    // Skip whitespace to check for #define
    this.skipWhitespace();

    if (this.check(TokenType.DEFINE)) {
      return this.parseMacroDefinition();
    }

    // Backtrack to preserve leading whitespace for parseCodeLine
    this.current = savedPosition;
    return this.parseCodeLine();
  }

  private parseMacroDefinition(): MacroDefinitionNode | null {
    const start = this.peek().position.start;
    const startLine = this.peek().position.line;
    const startColumn = this.peek().position.column;

    this.consume(TokenType.DEFINE);
    this.skipWhitespace();

    if (!this.check(TokenType.IDENTIFIER)) {
      this.addError('Expected macro name after #define', this.peek().position);
      this.synchronize();
      return null;
    }

    const nameToken = this.advance();
    const name = nameToken.value;

    // Add macro definition token
    this.macroTokens.push({
      type: 'macro_definition',
      range: {
        start: start,
        end:
          this.current < this.tokens.length
            ? this.peek().position.start
            : nameToken.position.end,
      },
      name,
    });

    let parameters: string[] | undefined;

    // Check for parameters
    if (this.check(TokenType.LPAREN)) {
      this.advance(); // consume (
      parameters = this.parseParameterList();

      if (!this.consume(TokenType.RPAREN)) {
        this.addError('Expected ) after parameter list', this.peek().position);
      }
    }

    this.skipWhitespace();

    // Check if this is a brace-style multiline macro
    let body: BodyNode[];
    if (this.check(TokenType.LBRACE)) {
      body = this.parseBraceMacroBody();
    } else {
      // Parse body (everything until newline, handling line continuations)
      body = this.parseMacroBody();
    }

    const end = this.previous().position.end;

    return {
      type: 'MacroDefinition',
      name,
      parameters,
      body,
      position: {
        start,
        end,
        line: startLine,
        column: startColumn,
      },
    };
  }

  private parseParameterList(): string[] {
    const params: string[] = [];

    this.skipWhitespace();

    if (!this.check(TokenType.RPAREN)) {
      do {
        this.skipWhitespace();

        if (!this.check(TokenType.IDENTIFIER)) {
          this.addError('Expected parameter name', this.peek().position);
          break;
        }

        params.push(this.advance().value);
        this.skipWhitespace();
      } while (this.match(TokenType.COMMA));
    }

    return params;
  }

  private parseMacroBody(): BodyNode[] {
    const body: BodyNode[] = [];

    // For multiline macros, we need to handle line continuations
    let hasLineContinuation = true;

    while (hasLineContinuation && !this.isAtEnd()) {
      hasLineContinuation = false;

      // Parse content until newline or backslash
      while (
        !this.isAtEnd() &&
        !this.check(TokenType.NEWLINE) &&
        !this.check(TokenType.BACKSLASH)
      ) {
        const content = this.parseContent();
        if (content) {
          body.push(content);
        }
      }

      // Check for line continuation
      if (this.check(TokenType.BACKSLASH)) {
        this.advance(); // consume backslash
        // The lexer already handles \n after backslash, so we just continue
        hasLineContinuation = true;

        // Skip any whitespace after the line continuation
        this.skipWhitespace();
      }
    }

    // Consume the final newline if present
    this.match(TokenType.NEWLINE);

    return body;
  }

  private parseBraceMacroBody(): BodyNode[] {
    const body: BodyNode[] = [];

    // Consume the opening brace
    this.consume(TokenType.LBRACE);

    // Skip any whitespace or comments after the opening brace
    this.skipWhitespaceAndNewlines();

    // Track brace depth for nested structures
    let braceDepth = 1;

    while (!this.isAtEnd() && braceDepth > 0) {
      // Skip whitespace and newlines at the beginning of each iteration
      while (
        this.match(TokenType.WHITESPACE) ||
        this.match(TokenType.NEWLINE)
      ) {
        // Continue
      }

      // Skip comments
      if (
        this.match(TokenType.COMMENT_SINGLE) ||
        this.match(TokenType.COMMENT_MULTI)
      ) {
        continue;
      }

      // Check for closing brace
      if (this.check(TokenType.RBRACE)) {
        braceDepth--;
        if (braceDepth === 0) {
          // Consume the final closing brace
          this.advance();
          break;
        }
        // It's a nested closing brace, include it as text
        const token = this.advance();
        body.push({
          type: 'Text',
          value: token.value,
          position: token.position,
        });
      } else if (this.check(TokenType.LBRACE)) {
        // Check if it's a builtin function
        const savedPosition = this.current;
        const content = this.parseContent();

        if (content && content.type === 'BuiltinFunction') {
          // It was parsed as a builtin function, add it
          body.push(content);
        } else {
          // It's a standalone opening brace
          // Reset position and consume as text
          this.current = savedPosition;
          const token = this.advance();
          body.push({
            type: 'Text',
            value: token.value,
            position: token.position,
          });
          braceDepth++;
        }
      } else {
        // Parse regular content
        const content = this.parseContent();
        if (content) {
          body.push(content);
        }
      }
    }

    if (braceDepth > 0) {
      this.addError(
        'Unclosed macro body - missing closing brace }',
        this.peek().position,
      );
    }

    return body;
  }

  private skipWhitespaceAndNewlines(): void {
    while (
      this.match(TokenType.WHITESPACE) ||
      this.match(TokenType.NEWLINE) ||
      this.match(TokenType.COMMENT_SINGLE) ||
      this.match(TokenType.COMMENT_MULTI)
    ) {
      // Continue
    }
  }

  private parseCodeLine(): CodeLineNode {
    const start = this.peek().position.start;
    const startLine = this.peek().position.line;
    const startColumn = this.peek().position.column;
    const content: ContentNode[] = [];

    // Preserve leading whitespace
    if (this.check(TokenType.WHITESPACE)) {
      const ws = this.advance();
      content.push({
        type: 'Text',
        value: ws.value,
        position: ws.position,
      });
    }

    while (!this.isAtEnd() && !this.check(TokenType.NEWLINE)) {
      const node = this.parseContent();
      if (node) {
        content.push(node);
      }
    }

    // Consume newline if present
    this.match(TokenType.NEWLINE);

    const end = this.previous().position.end;

    return {
      type: 'CodeLine',
      content,
      position: {
        start,
        end,
        line: startLine,
        column: startColumn,
      },
    };
  }

  private parseContent(): ContentNode | null {
    // Check for macro invocation (@ or #)
    if (this.check(TokenType.AT) || this.check(TokenType.HASH)) {
      return this.parseMacroInvocation();
    }

    // Check for builtin functions
    if (
      this.check(TokenType.BUILTIN_REPEAT) ||
      this.check(TokenType.BUILTIN_IF) ||
      this.check(TokenType.BUILTIN_FOR) ||
      this.check(TokenType.BUILTIN_REVERSE)
    ) {
      return this.parseBuiltinFunction();
    }

    // Check for Brainfuck commands
    if (this.check(TokenType.BF_COMMAND)) {
      return this.parseBrainfuckCommand();
    }

    // Check for whitespace
    if (this.check(TokenType.WHITESPACE)) {
      const token = this.advance();
      return {
        type: 'Text',
        value: token.value,
        position: token.position,
      };
    }

    // Handle other tokens as text
    const token = this.advance();
    return {
      type: 'Text',
      value: token.value,
      position: token.position,
    };
  }

  private parseMacroInvocation(): MacroInvocationNode | null {
    const start = this.peek().position.start;
    const startPos = this.peek().position;

    // Consume either @ or #
    const prefix = this.peek().value;
    if (this.check(TokenType.AT)) {
      this.consume(TokenType.AT);
    } else if (this.check(TokenType.HASH)) {
      this.consume(TokenType.HASH);
    }

    if (!this.check(TokenType.IDENTIFIER)) {
      // Not a valid macro invocation, return as text
      return {
        type: 'Text',
        value: prefix,
        position: startPos,
      } as any;
    }

    const nameToken = this.advance();
    const name = nameToken.value;
    let args: ExpressionNode[] | undefined;
    let end = nameToken.position.end;

    // Check for arguments
    if (this.check(TokenType.LPAREN)) {
      this.advance(); // consume (
      args = this.parseArgumentList();

      if (this.consume(TokenType.RPAREN)) {
        end = this.previous().position.end;
      } else {
        this.addError('Expected ) after arguments', this.peek().position);
      }
    }

    // Add macro invocation token
    this.macroTokens.push({
      type: 'macro_invocation',
      range: {
        start,
        end,
      },
      name,
    });

    return {
      type: 'MacroInvocation',
      name,
      arguments: args,
      position: {
        start,
        end,
        line: startPos.line,
        column: startPos.column,
      },
    };
  }

  private parseBuiltinFunction(): BuiltinFunctionNode | null {
    const start = this.peek().position.start;
    const startPos = this.peek().position;
    const functionToken = this.advance();
    let name: string;

    switch (functionToken.type) {
      case TokenType.BUILTIN_REPEAT:
        name = 'repeat';
        break;
      case TokenType.BUILTIN_IF:
        name = 'if';
        break;
      case TokenType.BUILTIN_FOR:
        name = 'for';
        break;
      case TokenType.BUILTIN_REVERSE:
        name = 'reverse';
        break;
      default:
        this.addError('Unknown builtin function', functionToken.position);
        return null;
    }

    if (!this.consume(TokenType.LPAREN)) {
      this.addError(`Expected ( after {${name}`, this.peek().position);
      return null;
    }

    let args: ExpressionNode[];

    if (name === 'for') {
      // Special parsing for for loop: {for(var in {values}, body)}
      args = this.parseForArguments();
    } else {
      args = this.parseArgumentList();
    }

    if (!this.consume(TokenType.RPAREN)) {
      this.addError('Expected ) after arguments', this.peek().position);
      return null;
    }

    if (!this.consume(TokenType.RBRACE)) {
      this.addError(
        'Expected } to close builtin function',
        this.peek().position,
      );
      return null;
    }

    const end = this.previous().position.end;

    // Add builtin function token
    this.macroTokens.push({
      type: 'builtin_function',
      range: {
        start,
        end,
      },
      name,
    });

    return {
      type: 'BuiltinFunction',
      name,
      arguments: args,
      position: {
        start,
        end,
        line: startPos.line,
        column: startPos.column,
      },
    };
  }

  private parseArgumentList(): ExpressionNode[] {
    const args: ExpressionNode[] = [];

    this.skipWhitespace();

    if (!this.check(TokenType.RPAREN)) {
      do {
        this.skipWhitespace();
        const arg = this.parseExpression();
        if (arg) {
          args.push(arg);
        }
        this.skipWhitespace();
      } while (this.match(TokenType.COMMA));
    }

    return args;
  }

  private parseForArguments(): ExpressionNode[] {
    const args: ExpressionNode[] = [];

    this.skipWhitespace();

    // Parse variable name or tuple pattern
    if (this.check(TokenType.LPAREN)) {
      // Parse tuple pattern: (a, b, c, ...)
      const start = this.peek().position.start;
      const startPos = this.peek().position;
      this.advance(); // consume (

      const elements: string[] = [];

      this.skipWhitespace();
      while (!this.check(TokenType.RPAREN) && !this.isAtEnd()) {
        if (!this.check(TokenType.IDENTIFIER)) {
          this.addError(
            'Expected identifier in tuple pattern',
            this.peek().position,
          );
          break;
        }

        const ident = this.advance();
        elements.push(ident.value);

        this.skipWhitespace();
        if (!this.check(TokenType.RPAREN)) {
          if (!this.consume(TokenType.COMMA)) {
            this.addError(
              'Expected , or ) in tuple pattern',
              this.peek().position,
            );
            break;
          }
          this.skipWhitespace();
        }
      }

      if (!this.consume(TokenType.RPAREN)) {
        this.addError(
          'Expected ) to close tuple pattern',
          this.peek().position,
        );
      }

      const end = this.previous().position.end;
      args.push({
        type: 'TuplePattern',
        elements,
        position: {
          start,
          end,
          line: startPos.line,
          column: startPos.column,
        },
      });
    } else if (this.check(TokenType.IDENTIFIER)) {
      // Single variable
      const varName = this.advance();
      args.push({
        type: 'Identifier',
        name: varName.value,
        position: varName.position,
      });
    } else {
      this.addError(
        'Expected variable name or tuple pattern in for loop',
        this.peek().position,
      );
      return args;
    }

    this.skipWhitespace();

    // Expect 'in' keyword
    if (!this.consume(TokenType.IN)) {
      this.addError('Expected "in" keyword in for loop', this.peek().position);
      return args;
    }

    this.skipWhitespace();

    // Parse array literal or expression
    // Special case: if it's a single identifier, parse it as an Identifier node
    if (this.check(TokenType.IDENTIFIER)) {
      const ident = this.advance();
      args.push({
        type: 'Identifier',
        name: ident.value,
        position: ident.position,
      });
    } else {
      const arrayExpr = this.parseArrayLiteral() || this.parseExpression();
      if (arrayExpr) {
        args.push(arrayExpr);
      }
    }

    this.skipWhitespace();

    // Expect comma
    if (!this.consume(TokenType.COMMA)) {
      this.addError(
        'Expected comma after array in for loop',
        this.peek().position,
      );
      return args;
    }

    this.skipWhitespace();

    // Parse body expression
    const body = this.parseExpression();
    if (body) {
      args.push(body);
    }

    return args;
  }

  private parseArrayLiteral(): ExpressionNode | null {
    if (!this.check(TokenType.LBRACE)) {
      return null;
    }

    const start = this.peek().position.start;
    const startPos = this.peek().position;
    this.advance(); // consume {

    const elements: ExpressionNode[] = [];

    this.skipWhitespace();

    while (!this.isAtEnd() && !this.check(TokenType.RBRACE)) {
      const element = this.parseExpression();
      if (element) {
        elements.push(element);
      }

      this.skipWhitespace();

      if (!this.check(TokenType.RBRACE)) {
        if (!this.consume(TokenType.COMMA)) {
          this.addError(
            'Expected comma or } in array literal',
            this.peek().position,
          );
          break;
        }
        this.skipWhitespace();
      }
    }

    if (!this.consume(TokenType.RBRACE)) {
      this.addError('Expected } to close array literal', this.peek().position);
      return null;
    }

    const end = this.previous().position.end;

    // Return array as a special expression node
    return {
      type: 'ArrayLiteral',
      elements,
      position: {
        start,
        end,
        line: startPos.line,
        column: startPos.column,
      },
    } as ArrayLiteralNode;
  }

  private parseExpression(): ExpressionNode | null {
    // Check for array literal first
    const arrayLiteral = this.parseArrayLiteral();
    if (arrayLiteral) {
      return arrayLiteral;
    }

    // This collects everything until a comma or closing paren
    const expressions: ContentNode[] = [];
    const start = this.peek().position.start;
    const startPos = this.peek().position;
    let parenDepth = 0;

    // Track raw text for simple arguments
    let rawText = '';
    let isSimpleText = true;

    while (!this.isAtEnd()) {
      const token = this.peek();

      if (token.type === TokenType.LPAREN) {
        if (parenDepth === 0 && expressions.length > 0) {
          // This might be a function call, let parseContent handle it
          const content = this.parseContent();
          if (content) {
            expressions.push(content);
            isSimpleText = false;
          }
          continue;
        }
        parenDepth++;
        rawText += token.value;
        this.advance();
      } else if (token.type === TokenType.RPAREN) {
        if (parenDepth === 0) break;
        parenDepth--;
        rawText += token.value;
        this.advance();
      } else if (token.type === TokenType.COMMA && parenDepth === 0) {
        break;
      } else if (token.type === TokenType.RBRACE && parenDepth === 0) {
        // Also break on closing brace for array literals
        break;
      } else if (
        token.type === TokenType.WHITESPACE ||
        token.type === TokenType.NEWLINE
      ) {
        // Skip whitespace in expressions
        rawText += ' ';
        this.advance();
      } else {
        // Let parseContent handle complex content
        const beforeCount = expressions.length;
        const content = this.parseContent();
        if (content) {
          expressions.push(content);
          if (content.type !== 'Text' || beforeCount > 0) {
            isSimpleText = false;
          } else if (content.type === 'Text') {
            rawText += (content as TextNode).value;
          }
        }
      }
    }

    if (expressions.length === 0 && rawText.trim() === '') return null;

    // For simple text arguments, return as a single text node
    if (isSimpleText && rawText.trim()) {
      const trimmed = rawText.trim();
      // Try to parse as number (including hexadecimal and character literals)
      let num: number;
      if (trimmed.match(/^'.*'$/)) {
        // Character literal - handle escape sequences
        const content = trimmed.slice(1, -1); // Remove quotes
        if (content.startsWith('\\') && content.length === 2) {
          // Escape sequence
          switch (content[1]) {
            case 'n':
              num = 10;
              break; // newline
            case 't':
              num = 9;
              break; // tab
            case 'r':
              num = 13;
              break; // carriage return
            case '\\':
              num = 92;
              break; // backslash
            case "'":
              num = 39;
              break; // single quote
            case '0':
              num = 0;
              break; // null
            default:
              num = content.charCodeAt(1); // Unknown escape, use literal
          }
        } else {
          // Regular character
          num = content.charCodeAt(0);
        }
      } else if (trimmed.startsWith('0x') || trimmed.startsWith('0X')) {
        num = parseInt(trimmed, 16);
      } else {
        num = parseInt(trimmed, 10);
      }
      if (
        !isNaN(num) &&
        (trimmed === num.toString() ||
          trimmed.toLowerCase() === '0x' + num.toString(16) ||
          trimmed.match(/^'.*'$/))
      ) {
        return {
          type: 'Number',
          value: num,
          position: {
            start,
            end: this.previous().position.end,
            line: startPos.line,
            column: startPos.column,
          },
        };
      }
      // Otherwise return as text for literal content
      return {
        type: 'Text',
        value: trimmed,
        position: {
          start,
          end: this.previous().position.end,
          line: startPos.line,
          column: startPos.column,
        },
      } as any;
    }

    if (expressions.length === 1) {
      // Special handling for single expressions
      const expr = expressions[0];
      if (expr.type === 'Text') {
        // Try to parse as number (including hexadecimal and character literals)
        const trimmed = (expr as TextNode).value.trim();
        let num: number;
        if (trimmed.match(/^'.*'$/)) {
          // Character literal - handle escape sequences
          const content = trimmed.slice(1, -1); // Remove quotes
          if (content.startsWith('\\') && content.length === 2) {
            // Escape sequence
            switch (content[1]) {
              case 'n':
                num = 10;
                break; // newline
              case 't':
                num = 9;
                break; // tab
              case 'r':
                num = 13;
                break; // carriage return
              case '\\':
                num = 92;
                break; // backslash
              case "'":
                num = 39;
                break; // single quote
              case '0':
                num = 0;
                break; // null
              default:
                num = content.charCodeAt(1); // Unknown escape, use literal
            }
          } else {
            // Regular character
            num = content.charCodeAt(0);
          }
        } else if (trimmed.startsWith('0x') || trimmed.startsWith('0X')) {
          num = parseInt(trimmed, 16);
        } else {
          num = parseInt(trimmed, 10);
        }
        if (
          !isNaN(num) &&
          (trimmed === num.toString() ||
            trimmed.toLowerCase() === '0x' + num.toString(16) ||
            trimmed.match(/^'.*'$/))
        ) {
          return {
            type: 'Number',
            value: num,
            position: expr.position,
          };
        }
        // Keep as text for things like "+" in repeat
        return expr as any;
      }
      return expr as ExpressionNode;
    }

    // Multiple expressions, wrap in ExpressionList
    const end = this.previous().position.end;
    return {
      type: 'ExpressionList',
      expressions,
      position: {
        start,
        end,
        line: startPos.line,
        column: startPos.column,
      },
    };
  }

  private parseBrainfuckCommand(): BrainfuckCommandNode {
    const token = this.advance();
    return {
      type: 'BrainfuckCommand',
      commands: token.value,
      position: token.position,
    };
  }

  // Helper methods
  private match(...types: TokenType[]): boolean {
    for (const type of types) {
      if (this.check(type)) {
        this.advance();
        return true;
      }
    }
    return false;
  }

  private check(type: TokenType): boolean {
    if (this.isAtEnd()) return false;
    return this.peek().type === type;
  }

  private advance(): Token {
    if (!this.isAtEnd()) this.current++;
    return this.previous();
  }

  private isAtEnd(): boolean {
    return (
      this.current >= this.tokens.length || this.peek().type === TokenType.EOF
    );
  }

  private peek(): Token {
    return this.tokens[this.current];
  }

  private previous(): Token {
    return this.tokens[this.current - 1];
  }

  private consume(type: TokenType): boolean {
    if (this.check(type)) {
      this.advance();
      return true;
    }
    return false;
  }

  private skipWhitespace(): void {
    while (this.match(TokenType.WHITESPACE)) {
      // Continue
    }
  }

  private synchronize(): void {
    // Skip to next statement
    while (!this.isAtEnd()) {
      if (this.previous().type === TokenType.NEWLINE) return;

      switch (this.peek().type) {
        case TokenType.DEFINE:
          return;
      }

      this.advance();
    }
  }

  private addError(
    message: string,
    position: { line: number; column: number; start: number; end: number },
  ): void {
    this.errors.push({
      type: 'syntax_error',
      message,
      location: {
        line: position.line - 1,
        column: position.column - 1,
        length: position.end - position.start,
      },
    });
  }
}

// Helper function to parse with lexing
export function parseMacro(
  input: string,
  lexerOptions?: LexerOptions,
): ParseResult {
  const tokens = tokenize(input, {
    preserveWhitespace: true,
    preserveComments: false,
    ...lexerOptions,
  });
  const parser = new MacroParser(tokens);
  return parser.parse();
}
