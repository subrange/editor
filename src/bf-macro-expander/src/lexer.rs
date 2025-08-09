use crate::ast::ASTPosition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // Literals
    Identifier,
    Number,
    Text,
    BrainfuckCommand,
    
    // Keywords
    Define,
    In,
    
    // Builtin functions
    BuiltinRepeat,
    BuiltinIf,
    BuiltinFor,
    BuiltinReverse,
    
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    At,
    Hash,
    Backslash,
    
    // Special
    Newline,
    Whitespace,
    Comment,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub position: ASTPosition,
}

pub struct Lexer {
    input: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    preserve_comments: bool,
}

impl Lexer {
    pub fn new(input: &str, preserve_comments: bool) -> Self {
        Self {
            input: input.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            preserve_comments,
        }
    }
    
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            if let Some(token) = self.scan_token() {
                tokens.push(token);
            }
        }
        
        tokens.push(Token {
            token_type: TokenType::Eof,
            value: String::new(),
            position: self.make_position(self.current, self.current),
        });
        
        tokens
    }
    
    fn scan_token(&mut self) -> Option<Token> {
        let start = self.current;
        let _start_line = self.line;
        let _start_column = self.column;
        
        let ch = self.advance();
        
        match ch {
            '\n' => {
                self.line += 1;
                self.column = 1;
                Some(self.make_token(TokenType::Newline, "\n", start))
            }
            ' ' | '\t' | '\r' => {
                while !self.is_at_end() && self.peek().is_whitespace() && self.peek() != '\n' {
                    self.advance();
                }
                let text = self.get_text(start);
                Some(self.make_token(TokenType::Whitespace, &text, start))
            }
            '#' => {
                // Check for #define
                if self.peek_string("define") {
                    self.advance_by(6);
                    Some(self.make_token(TokenType::Define, "#define", start))
                } else {
                    Some(self.make_token(TokenType::Hash, "#", start))
                }
            }
            '@' => Some(self.make_token(TokenType::At, "@", start)),
            '(' => Some(self.make_token(TokenType::LParen, "(", start)),
            ')' => Some(self.make_token(TokenType::RParen, ")", start)),
            '{' => {
                // Check for builtin functions
                if self.match_string("repeat") {
                    Some(self.make_token(TokenType::BuiltinRepeat, "{repeat", start))
                } else if self.match_string("if") {
                    Some(self.make_token(TokenType::BuiltinIf, "{if", start))
                } else if self.match_string("for") {
                    Some(self.make_token(TokenType::BuiltinFor, "{for", start))
                } else if self.match_string("reverse") {
                    Some(self.make_token(TokenType::BuiltinReverse, "{reverse", start))
                } else {
                    Some(self.make_token(TokenType::LBrace, "{", start))
                }
            }
            '}' => Some(self.make_token(TokenType::RBrace, "}", start)),
            ',' => Some(self.make_token(TokenType::Comma, ",", start)),
            '\\' => Some(self.make_token(TokenType::Backslash, "\\", start)),
            '/' => {
                if self.peek() == '/' {
                    // Line comment
                    self.advance();
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                    let text = self.get_text(start);
                    if self.preserve_comments {
                        Some(self.make_token(TokenType::Comment, &text, start))
                    } else {
                        None
                    }
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
                    if self.preserve_comments {
                        Some(self.make_token(TokenType::Comment, &text, start))
                    } else {
                        None
                    }
                } else {
                    Some(self.make_token(TokenType::Text, "/", start))
                }
            }
            '>' | '<' | '+' | '-' | '.' | '[' | ']' | '$' => {
                // Brainfuck commands
                while !self.is_at_end() && self.is_bf_command(self.peek()) {
                    self.advance();
                }
                let text = self.get_text(start);
                Some(self.make_token(TokenType::BrainfuckCommand, &text, start))
            }
            '\'' => {
                // Character literal
                if !self.is_at_end() {
                    self.advance(); // consume character
                    if !self.is_at_end() && self.peek() == '\'' {
                        self.advance(); // consume closing quote
                    }
                }
                let text = self.get_text(start);
                Some(self.make_token(TokenType::Text, &text, start))
            }
            '"' => {
                // String literal
                while !self.is_at_end() && self.peek() != '"' {
                    if self.peek() == '\\' {
                        self.advance(); // consume backslash
                        if !self.is_at_end() {
                            self.advance(); // consume escaped character
                        }
                    } else {
                        self.advance();
                    }
                }
                if !self.is_at_end() {
                    self.advance(); // consume closing quote
                }
                let text = self.get_text(start);
                Some(self.make_token(TokenType::Text, &text, start))
            }
            c if c.is_ascii_digit() => {
                // Number (including hex)
                if c == '0' && (self.peek() == 'x' || self.peek() == 'X') {
                    self.advance(); // consume 'x' or 'X'
                    while !self.is_at_end() && self.peek().is_ascii_hexdigit() {
                        self.advance();
                    }
                } else {
                    while !self.is_at_end() && self.peek().is_ascii_digit() {
                        self.advance();
                    }
                }
                let text = self.get_text(start);
                Some(self.make_token(TokenType::Number, &text, start))
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                // Identifier or keyword
                while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '_') {
                    self.advance();
                }
                let text = self.get_text(start);
                
                let token_type = if text == "in" {
                    TokenType::In
                } else {
                    TokenType::Identifier
                };
                
                Some(self.make_token(token_type, &text, start))
            }
            _ => {
                // Default to text for any other character
                Some(self.make_token(TokenType::Text, &ch.to_string(), start))
            }
        }
    }
    
    fn is_bf_command(&self, ch: char) -> bool {
        matches!(ch, '>' | '<' | '+' | '-' | '.' | '[' | ']' | '$')
    }
    
    fn advance(&mut self) -> char {
        let ch = self.input[self.current];
        self.current += 1;
        self.column += 1;
        ch
    }
    
    fn advance_by(&mut self, n: usize) {
        for _ in 0..n {
            if !self.is_at_end() {
                self.advance();
            }
        }
    }
    
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.current]
        }
    }
    
    fn match_string(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        
        // Check if we have enough characters left
        if self.current + chars.len() > self.input.len() {
            return false;
        }
        
        // Check if the characters match
        for (i, ch) in chars.iter().enumerate() {
            if self.input[self.current + i] != *ch {
                return false;
            }
        }
        
        // Advance past the matched string
        for _ in 0..chars.len() {
            self.advance();
        }
        
        true
    }
    
    fn peek_ahead(&self, n: usize) -> Option<char> {
        let pos = self.current + n;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }
    
    fn peek_string(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        for (i, &ch) in chars.iter().enumerate() {
            if self.current + i >= self.input.len() || self.input[self.current + i] != ch {
                return false;
            }
        }
        true
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.input.len()
    }
    
    fn get_text(&self, start: usize) -> String {
        self.input[start..self.current].iter().collect()
    }
    
    fn make_token(&self, token_type: TokenType, value: &str, start: usize) -> Token {
        Token {
            token_type,
            value: value.to_string(),
            position: self.make_position(start, self.current),
        }
    }
    
    fn make_position(&self, start: usize, end: usize) -> ASTPosition {
        ASTPosition {
            start,
            end,
            line: self.line,
            column: self.column,
        }
    }
}