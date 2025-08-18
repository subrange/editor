use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Preprocessor directives
    Hash,           // #
    Include,        // include
    Define,         // define
    Undef,          // undef
    If,             // if
    Ifdef,          // ifdef
    Ifndef,         // ifndef
    Elif,           // elif
    Else,           // else
    Endif,          // endif
    Line,           // line
    Pragma,         // pragma
    Error,          // error directive
    Warning,        // warning directive
    
    // Identifiers and literals
    Identifier(String),
    Number(String),
    StringLiteral(String),
    
    // Operators and punctuation
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Comma,          // ,
    Dot,            // .
    Arrow,          // ->
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    Ampersand,      // &
    Pipe,           // |
    Caret,          // ^
    Tilde,          // ~
    Bang,           // !
    Question,       // ?
    Colon,          // :
    Semicolon,      // ;
    Equal,          // =
    NotEqual,       // !=
    Less,           // <
    Greater,        // >
    LessEqual,      // <=
    GreaterEqual,   // >=
    LogicalAnd,     // &&
    LogicalOr,      // ||
    LeftShift,      // <<
    RightShift,     // >>
    PlusEqual,      // +=
    MinusEqual,     // -=
    StarEqual,      // *=
    SlashEqual,     // /=
    PercentEqual,   // %=
    AmpersandEqual, // &=
    PipeEqual,      // |=
    CaretEqual,     // ^=
    LeftShiftEqual, // <<=
    RightShiftEqual,// >>=
    DoubleHash,     // ## (token paste)
    Ellipsis,       // ...
    
    // Special
    Newline,
    Whitespace(String),
    Comment(String),
    Text(String),   // Regular C code text
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub text: String,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {
    input: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            if let Some(token) = self.scan_token()? {
                tokens.push(token);
            }
        }
        
        tokens.push(Token {
            token_type: TokenType::Eof,
            text: String::new(),
            line: self.line,
            column: self.column,
        });
        
        Ok(tokens)
    }
    
    fn scan_token(&mut self) -> Result<Option<Token>> {
        let start_line = self.line;
        let start_column = self.column;
        let start = self.current;
        
        let ch = self.advance();
        
        let token = match ch {
            '\n' => {
                self.line += 1;
                self.column = 1;
                Some(self.make_token(TokenType::Newline, "\n", start_line, start_column))
            }
            ' ' | '\t' | '\r' => {
                while !self.is_at_end() && self.peek().is_whitespace() && self.peek() != '\n' {
                    self.advance();
                }
                let text = self.get_text(start);
                Some(self.make_token(TokenType::Whitespace(text.clone()), &text, start_line, start_column))
            }
            '#' => {
                if self.peek() == '#' {
                    self.advance();
                    Some(self.make_token(TokenType::DoubleHash, "##", start_line, start_column))
                } else {
                    // Check for preprocessor directive
                    self.scan_preprocessor_directive(start_line, start_column)
                }
            }
            '/' => {
                if self.peek() == '/' {
                    // Line comment
                    self.advance();
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                    let text = self.get_text(start);
                    Some(self.make_token(TokenType::Comment(text.clone()), &text, start_line, start_column))
                } else if self.peek() == '*' {
                    // Block comment
                    self.advance();
                    while !self.is_at_end() {
                        if self.peek() == '*' && self.peek_ahead(1) == Some('/') {
                            self.advance();
                            self.advance();
                            break;
                        }
                        if self.peek() == '\n' {
                            self.line += 1;
                            self.column = 0;
                        }
                        self.advance();
                    }
                    let text = self.get_text(start);
                    Some(self.make_token(TokenType::Comment(text.clone()), &text, start_line, start_column))
                } else if self.peek() == '=' {
                    self.advance();
                    Some(self.make_token(TokenType::SlashEqual, "/=", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Slash, "/", start_line, start_column))
                }
            }
            '"' => {
                // String literal - preserve everything including escape sequences
                let mut text = String::new();
                while !self.is_at_end() && self.peek() != '"' {
                    if self.peek() == '\\' {
                        text.push(self.advance()); // Add the backslash
                        if !self.is_at_end() {
                            text.push(self.advance()); // Add the escaped character
                        }
                    } else {
                        if self.peek() == '\n' {
                            self.line += 1;
                            self.column = 0;
                        }
                        text.push(self.advance());
                    }
                }
                if !self.is_at_end() {
                    self.advance(); // closing quote
                }
                Some(self.make_token(TokenType::StringLiteral(text.clone()), &format!("\"{}\"", text), start_line, start_column))
            }
            '\'' => {
                // Character literal - preserve the entire literal including escape sequences
                let mut literal = String::from("'");
                while !self.is_at_end() && self.peek() != '\'' {
                    if self.peek() == '\\' {
                        literal.push(self.advance()); // backslash
                        if !self.is_at_end() {
                            literal.push(self.advance()); // escaped character
                        }
                    } else {
                        literal.push(self.advance());
                    }
                }
                if !self.is_at_end() && self.peek() == '\'' {
                    self.advance();
                    literal.push('\'');
                }
                // Use Text token type to preserve the literal as-is
                Some(self.make_token(TokenType::Text(literal.clone()), &literal, start_line, start_column))
            }
            '(' => Some(self.make_token(TokenType::LeftParen, "(", start_line, start_column)),
            ')' => Some(self.make_token(TokenType::RightParen, ")", start_line, start_column)),
            '{' => Some(self.make_token(TokenType::LeftBrace, "{", start_line, start_column)),
            '}' => Some(self.make_token(TokenType::RightBrace, "}", start_line, start_column)),
            '[' => Some(self.make_token(TokenType::LeftBracket, "[", start_line, start_column)),
            ']' => Some(self.make_token(TokenType::RightBracket, "]", start_line, start_column)),
            ',' => Some(self.make_token(TokenType::Comma, ",", start_line, start_column)),
            ';' => Some(self.make_token(TokenType::Semicolon, ";", start_line, start_column)),
            ':' => Some(self.make_token(TokenType::Colon, ":", start_line, start_column)),
            '?' => Some(self.make_token(TokenType::Question, "?", start_line, start_column)),
            '~' => Some(self.make_token(TokenType::Tilde, "~", start_line, start_column)),
            '.' => {
                if self.peek() == '.' && self.peek_ahead(1) == Some('.') {
                    self.advance();
                    self.advance();
                    Some(self.make_token(TokenType::Ellipsis, "...", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Dot, ".", start_line, start_column))
                }
            }
            '+' => {
                if self.peek() == '=' {
                    self.advance();
                    Some(self.make_token(TokenType::PlusEqual, "+=", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Plus, "+", start_line, start_column))
                }
            }
            '-' => {
                if self.peek() == '>' {
                    self.advance();
                    Some(self.make_token(TokenType::Arrow, "->", start_line, start_column))
                } else if self.peek() == '=' {
                    self.advance();
                    Some(self.make_token(TokenType::MinusEqual, "-=", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Minus, "-", start_line, start_column))
                }
            }
            '*' => {
                if self.peek() == '=' {
                    self.advance();
                    Some(self.make_token(TokenType::StarEqual, "*=", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Star, "*", start_line, start_column))
                }
            }
            '%' => {
                if self.peek() == '=' {
                    self.advance();
                    Some(self.make_token(TokenType::PercentEqual, "%=", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Percent, "%", start_line, start_column))
                }
            }
            '&' => {
                if self.peek() == '&' {
                    self.advance();
                    Some(self.make_token(TokenType::LogicalAnd, "&&", start_line, start_column))
                } else if self.peek() == '=' {
                    self.advance();
                    Some(self.make_token(TokenType::AmpersandEqual, "&=", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Ampersand, "&", start_line, start_column))
                }
            }
            '|' => {
                if self.peek() == '|' {
                    self.advance();
                    Some(self.make_token(TokenType::LogicalOr, "||", start_line, start_column))
                } else if self.peek() == '=' {
                    self.advance();
                    Some(self.make_token(TokenType::PipeEqual, "|=", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Pipe, "|", start_line, start_column))
                }
            }
            '^' => {
                if self.peek() == '=' {
                    self.advance();
                    Some(self.make_token(TokenType::CaretEqual, "^=", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Caret, "^", start_line, start_column))
                }
            }
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    Some(self.make_token(TokenType::NotEqual, "!=", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Bang, "!", start_line, start_column))
                }
            }
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    Some(self.make_token(TokenType::Equal, "==", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Equal, "=", start_line, start_column))
                }
            }
            '<' => {
                if self.peek() == '<' {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        Some(self.make_token(TokenType::LeftShiftEqual, "<<=", start_line, start_column))
                    } else {
                        Some(self.make_token(TokenType::LeftShift, "<<", start_line, start_column))
                    }
                } else if self.peek() == '=' {
                    self.advance();
                    Some(self.make_token(TokenType::LessEqual, "<=", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Less, "<", start_line, start_column))
                }
            }
            '>' => {
                if self.peek() == '>' {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        Some(self.make_token(TokenType::RightShiftEqual, ">>=", start_line, start_column))
                    } else {
                        Some(self.make_token(TokenType::RightShift, ">>", start_line, start_column))
                    }
                } else if self.peek() == '=' {
                    self.advance();
                    Some(self.make_token(TokenType::GreaterEqual, ">=", start_line, start_column))
                } else {
                    Some(self.make_token(TokenType::Greater, ">", start_line, start_column))
                }
            }
            c if c.is_ascii_digit() => {
                // Number
                while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '.') {
                    self.advance();
                }
                let text = self.get_text(start);
                Some(self.make_token(TokenType::Number(text.clone()), &text, start_line, start_column))
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                // Identifier
                while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '_') {
                    self.advance();
                }
                let text = self.get_text(start);
                Some(self.make_token(TokenType::Identifier(text.clone()), &text, start_line, start_column))
            }
            _ => {
                // Unknown character, treat as text
                Some(self.make_token(TokenType::Text(ch.to_string()), &ch.to_string(), start_line, start_column))
            }
        };
        
        Ok(token)
    }
    
    fn scan_preprocessor_directive(&mut self, start_line: usize, start_column: usize) -> Option<Token> {
        // Skip whitespace after #
        while !self.is_at_end() && (self.peek() == ' ' || self.peek() == '\t') {
            self.advance();
        }
        
        // Read directive name
        let directive_start = self.current;
        while !self.is_at_end() && self.peek().is_ascii_alphabetic() {
            self.advance();
        }
        
        if self.current == directive_start {
            // Just a # without directive
            return Some(self.make_token(TokenType::Hash, "#", start_line, start_column));
        }
        
        let directive = self.get_text(directive_start);
        let token_type = match directive.as_str() {
            "include" => TokenType::Include,
            "define" => TokenType::Define,
            "undef" => TokenType::Undef,
            "if" => TokenType::If,
            "ifdef" => TokenType::Ifdef,
            "ifndef" => TokenType::Ifndef,
            "elif" => TokenType::Elif,
            "else" => TokenType::Else,
            "endif" => TokenType::Endif,
            "line" => TokenType::Line,
            "pragma" => TokenType::Pragma,
            "error" => TokenType::Error,
            "warning" => TokenType::Warning,
            _ => TokenType::Hash, // Unknown directive
        };
        
        Some(self.make_token(token_type, &format!("#{}", directive), start_line, start_column))
    }
    
    fn advance(&mut self) -> char {
        let ch = self.input[self.current];
        self.current += 1;
        self.column += 1;
        ch
    }
    
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.current]
        }
    }
    
    fn peek_ahead(&self, n: usize) -> Option<char> {
        let pos = self.current + n;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.input.len()
    }
    
    fn get_text(&self, start: usize) -> String {
        self.input[start..self.current].iter().collect()
    }
    
    fn make_token(&self, token_type: TokenType, text: &str, line: usize, column: usize) -> Token {
        Token {
            token_type,
            text: text.to_string(),
            line,
            column,
        }
    }
}

/// Convenience function to tokenize input
pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut lexer = Lexer::new(input);
    lexer.tokenize()
}