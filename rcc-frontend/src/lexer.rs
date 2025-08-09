//! C99 Lexer
//! 
//! Tokenizes C99 source code into a stream of tokens.
//! Handles keywords, operators, literals, identifiers, and comments.

use rcc_common::{SourceLocation, SourceSpan, CompilerError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// C99 Token types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    // Literals
    IntLiteral(i64),
    CharLiteral(u8),
    StringLiteral(String),
    
    // Identifiers and keywords
    Identifier(String),
    
    // Keywords
    Auto, Break, Case, Char, Const, Continue, Default, Do,
    Double, Else, Enum, Extern, Float, For, Goto, If,
    Int, Long, Register, Return, Short, Signed, Sizeof, Static,
    Struct, Switch, Typedef, Union, Unsigned, Void, Volatile, While,
    
    // Operators
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
    Equal,          // =
    Less,           // <
    Greater,        // >
    Question,       // ?
    Colon,          // :
    
    // Compound operators
    PlusPlus,       // ++
    MinusMinus,     // --
    LeftShift,      // <<
    RightShift,     // >>
    LessEqual,      // <=
    GreaterEqual,   // >=
    EqualEqual,     // ==
    BangEqual,      // !=
    AmpersandAmpersand, // &&
    PipePipe,       // ||
    
    // Assignment operators
    PlusEqual,      // +=
    MinusEqual,     // -=
    StarEqual,      // *=
    SlashEqual,     // /=
    PercentEqual,   // %=
    AmpersandEqual, // &=
    PipeEqual,      // |=
    CaretEqual,     // ^=
    LeftShiftEqual, // <<=
    RightShiftEqual, // >>=
    
    // Delimiters
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Semicolon,      // ;
    Comma,          // ,
    Dot,            // .
    Arrow,          // ->
    
    // Special
    Newline,
    EndOfFile,
    
    // Comments (optional - may be stripped)
    LineComment(String),
    BlockComment(String),
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::IntLiteral(n) => write!(f, "{}", n),
            TokenType::CharLiteral(c) => write!(f, "'{}'", *c as char),
            TokenType::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenType::Identifier(s) => write!(f, "{}", s),
            
            // Keywords
            TokenType::Auto => write!(f, "auto"),
            TokenType::Break => write!(f, "break"),
            TokenType::Case => write!(f, "case"),
            TokenType::Char => write!(f, "char"),
            TokenType::Const => write!(f, "const"),
            TokenType::Continue => write!(f, "continue"),
            TokenType::Default => write!(f, "default"),
            TokenType::Do => write!(f, "do"),
            TokenType::Double => write!(f, "double"),
            TokenType::Else => write!(f, "else"),
            TokenType::Enum => write!(f, "enum"),
            TokenType::Extern => write!(f, "extern"),
            TokenType::Float => write!(f, "float"),
            TokenType::For => write!(f, "for"),
            TokenType::Goto => write!(f, "goto"),
            TokenType::If => write!(f, "if"),
            TokenType::Int => write!(f, "int"),
            TokenType::Long => write!(f, "long"),
            TokenType::Register => write!(f, "register"),
            TokenType::Return => write!(f, "return"),
            TokenType::Short => write!(f, "short"),
            TokenType::Signed => write!(f, "signed"),
            TokenType::Sizeof => write!(f, "sizeof"),
            TokenType::Static => write!(f, "static"),
            TokenType::Struct => write!(f, "struct"),
            TokenType::Switch => write!(f, "switch"),
            TokenType::Typedef => write!(f, "typedef"),
            TokenType::Union => write!(f, "union"),
            TokenType::Unsigned => write!(f, "unsigned"),
            TokenType::Void => write!(f, "void"),
            TokenType::Volatile => write!(f, "volatile"),
            TokenType::While => write!(f, "while"),
            
            // Operators - show the symbol
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Star => write!(f, "*"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Percent => write!(f, "%"),
            TokenType::Ampersand => write!(f, "&"),
            TokenType::Pipe => write!(f, "|"),
            TokenType::Caret => write!(f, "^"),
            TokenType::Tilde => write!(f, "~"),
            TokenType::Bang => write!(f, "!"),
            TokenType::Equal => write!(f, "="),
            TokenType::Less => write!(f, "<"),
            TokenType::Greater => write!(f, ">"),
            TokenType::Question => write!(f, "?"),
            TokenType::Colon => write!(f, ":"),
            
            TokenType::PlusPlus => write!(f, "++"),
            TokenType::MinusMinus => write!(f, "--"),
            TokenType::LeftShift => write!(f, "<<"),
            TokenType::RightShift => write!(f, ">>"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::BangEqual => write!(f, "!="),
            TokenType::AmpersandAmpersand => write!(f, "&&"),
            TokenType::PipePipe => write!(f, "||"),
            
            TokenType::PlusEqual => write!(f, "+="),
            TokenType::MinusEqual => write!(f, "-="),
            TokenType::StarEqual => write!(f, "*="),
            TokenType::SlashEqual => write!(f, "/="),
            TokenType::PercentEqual => write!(f, "%="),
            TokenType::AmpersandEqual => write!(f, "&="),
            TokenType::PipeEqual => write!(f, "|="),
            TokenType::CaretEqual => write!(f, "^="),
            TokenType::LeftShiftEqual => write!(f, "<<="),
            TokenType::RightShiftEqual => write!(f, ">>="),
            
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Comma => write!(f, ","),
            TokenType::Dot => write!(f, "."),
            TokenType::Arrow => write!(f, "->"),
            
            TokenType::Newline => write!(f, "\\n"),
            TokenType::EndOfFile => write!(f, "EOF"),
            TokenType::LineComment(s) => write!(f, "//{}", s),
            TokenType::BlockComment(s) => write!(f, "/*{}*/", s),
        }
    }
}

/// A token with location information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub span: SourceSpan,
}

impl Token {
    pub fn new(token_type: TokenType, span: SourceSpan) -> Self {
        Self { token_type, span }
    }
    
    pub fn eof(location: SourceLocation) -> Self {
        Self {
            token_type: TokenType::EndOfFile,
            span: SourceSpan::new(location.clone(), location),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.token_type, self.span.start)
    }
}

/// C99 Lexer
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: u32,
    column: u32,
    keywords: HashMap<String, TokenType>,
}

impl Lexer {
    /// Create a new lexer
    pub fn new(input: &str) -> Self {
        let mut lexer = Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            keywords: HashMap::new(),
        };
        
        lexer.initialize_keywords();
        lexer
    }
    
    /// Initialize keyword map
    fn initialize_keywords(&mut self) {
        let keywords = [
            ("auto", TokenType::Auto),
            ("break", TokenType::Break),
            ("case", TokenType::Case),
            ("char", TokenType::Char),
            ("const", TokenType::Const),
            ("continue", TokenType::Continue),
            ("default", TokenType::Default),
            ("do", TokenType::Do),
            ("double", TokenType::Double),
            ("else", TokenType::Else),
            ("enum", TokenType::Enum),
            ("extern", TokenType::Extern),
            ("float", TokenType::Float),
            ("for", TokenType::For),
            ("goto", TokenType::Goto),
            ("if", TokenType::If),
            ("int", TokenType::Int),
            ("long", TokenType::Long),
            ("register", TokenType::Register),
            ("return", TokenType::Return),
            ("short", TokenType::Short),
            ("signed", TokenType::Signed),
            ("sizeof", TokenType::Sizeof),
            ("static", TokenType::Static),
            ("struct", TokenType::Struct),
            ("switch", TokenType::Switch),
            ("typedef", TokenType::Typedef),
            ("union", TokenType::Union),
            ("unsigned", TokenType::Unsigned),
            ("void", TokenType::Void),
            ("volatile", TokenType::Volatile),
            ("while", TokenType::While),
        ];
        
        for (keyword, token_type) in keywords {
            self.keywords.insert(keyword.to_string(), token_type);
        }
    }
    
    /// Get current character
    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }
    
    /// Peek ahead n characters
    fn peek_char(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }
    
    /// Advance to next character
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.current_char() {
            self.position += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }
    
    /// Get current location
    fn current_location(&self) -> SourceLocation {
        SourceLocation::new_simple(self.line, self.column)
    }
    
    /// Skip whitespace (except newlines)
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    /// Tokenize an identifier or keyword
    fn tokenize_identifier(&mut self) -> Result<TokenType, CompilerError> {
        let mut identifier = String::new();
        
        // First character must be letter or underscore
        if let Some(ch) = self.current_char() {
            if ch.is_alphabetic() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                return Err(CompilerError::lexer_error(
                    format!("Invalid identifier start: {}", ch),
                    self.current_location(),
                ));
            }
        }
        
        // Subsequent characters can be letters, digits, or underscores
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check if it's a keyword
        if let Some(keyword_token) = self.keywords.get(&identifier) {
            Ok(keyword_token.clone())
        } else {
            Ok(TokenType::Identifier(identifier))
        }
    }
    
    /// Tokenize an integer literal
    fn tokenize_integer(&mut self) -> Result<TokenType, CompilerError> {
        let mut number = String::new();
        
        // Handle hex prefix
        if self.current_char() == Some('0') && self.peek_char(1) == Some('x') {
            number.push_str("0x");
            self.advance(); // '0'
            self.advance(); // 'x'
            
            while let Some(ch) = self.current_char() {
                if ch.is_ascii_hexdigit() {
                    number.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            
            if number.len() == 2 {
                return Err(CompilerError::lexer_error(
                    "Invalid hex literal".to_string(),
                    self.current_location(),
                ));
            }
            
            let value = i64::from_str_radix(&number[2..], 16)
                .map_err(|_| CompilerError::lexer_error(
                    format!("Invalid hex literal: {}", number),
                    self.current_location(),
                ))?;
            
            return Ok(TokenType::IntLiteral(value));
        }
        
        // Handle decimal numbers
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        let value = number.parse::<i64>()
            .map_err(|_| CompilerError::lexer_error(
                format!("Invalid integer literal: {}", number),
                self.current_location(),
            ))?;
        
        Ok(TokenType::IntLiteral(value))
    }
    
    /// Tokenize a character literal
    fn tokenize_char_literal(&mut self) -> Result<TokenType, CompilerError> {
        self.advance(); // Skip opening quote
        
        let ch = match self.current_char() {
            Some('\\') => {
                self.advance(); // Skip backslash
                match self.current_char() {
                    Some('n') => { self.advance(); b'\n' },
                    Some('t') => { self.advance(); b'\t' },
                    Some('r') => { self.advance(); b'\r' },
                    Some('\\') => { self.advance(); b'\\' },
                    Some('\'') => { self.advance(); b'\'' },
                    Some('0') => { self.advance(); 0 },
                    Some(c) => {
                        return Err(CompilerError::lexer_error(
                            format!("Invalid escape sequence: \\{}", c),
                            self.current_location(),
                        ));
                    }
                    None => {
                        return Err(CompilerError::lexer_error(
                            "Unterminated character literal".to_string(),
                            self.current_location(),
                        ));
                    }
                }
            }
            Some(ch) if ch != '\'' => {
                self.advance();
                ch as u8
            }
            _ => {
                return Err(CompilerError::lexer_error(
                    "Empty character literal".to_string(),
                    self.current_location(),
                ));
            }
        };
        
        if self.current_char() != Some('\'') {
            return Err(CompilerError::lexer_error(
                "Unterminated character literal".to_string(),
                self.current_location(),
            ));
        }
        
        self.advance(); // Skip closing quote
        Ok(TokenType::CharLiteral(ch))
    }
    
    /// Tokenize a string literal
    fn tokenize_string_literal(&mut self) -> Result<TokenType, CompilerError> {
        self.advance(); // Skip opening quote
        let mut string = String::new();
        
        while let Some(ch) = self.current_char() {
            match ch {
                '"' => {
                    self.advance();
                    return Ok(TokenType::StringLiteral(string));
                }
                '\\' => {
                    self.advance(); // Skip backslash
                    match self.current_char() {
                        Some('n') => { string.push('\n'); self.advance(); },
                        Some('t') => { string.push('\t'); self.advance(); },
                        Some('r') => { string.push('\r'); self.advance(); },
                        Some('\\') => { string.push('\\'); self.advance(); },
                        Some('"') => { string.push('"'); self.advance(); },
                        Some('0') => { string.push('\0'); self.advance(); },
                        Some(c) => {
                            return Err(CompilerError::lexer_error(
                                format!("Invalid escape sequence: \\{}", c),
                                self.current_location(),
                            ));
                        }
                        None => {
                            return Err(CompilerError::lexer_error(
                                "Unterminated string literal".to_string(),
                                self.current_location(),
                            ));
                        }
                    }
                }
                _ => {
                    string.push(ch);
                    self.advance();
                }
            }
        }
        
        Err(CompilerError::lexer_error(
            "Unterminated string literal".to_string(),
            self.current_location(),
        ))
    }
    
    /// Tokenize line comment
    fn tokenize_line_comment(&mut self) -> TokenType {
        self.advance(); // Skip first '/'
        self.advance(); // Skip second '/'
        
        let mut comment = String::new();
        while let Some(ch) = self.current_char() {
            if ch == '\n' {
                break;
            }
            comment.push(ch);
            self.advance();
        }
        
        TokenType::LineComment(comment)
    }
    
    /// Tokenize block comment
    fn tokenize_block_comment(&mut self) -> Result<TokenType, CompilerError> {
        self.advance(); // Skip '/'
        self.advance(); // Skip '*'
        
        let mut comment = String::new();
        let mut found_end = false;
        
        while let Some(ch) = self.current_char() {
            if ch == '*' && self.peek_char(1) == Some('/') {
                self.advance(); // Skip '*'
                self.advance(); // Skip '/'
                found_end = true;
                break;
            }
            comment.push(ch);
            self.advance();
        }
        
        if !found_end {
            return Err(CompilerError::lexer_error(
                "Unterminated block comment".to_string(),
                self.current_location(),
            ));
        }
        
        Ok(TokenType::BlockComment(comment))
    }
    
    /// Get next token
    pub fn next_token(&mut self) -> Result<Token, CompilerError> {
        self.skip_whitespace();
        
        let start_location = self.current_location();
        
        let token_type = match self.current_char() {
            None => TokenType::EndOfFile,
            
            Some('\n') => {
                self.advance();
                TokenType::Newline
            }
            
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                self.tokenize_identifier()?
            }
            
            Some(ch) if ch.is_ascii_digit() => {
                self.tokenize_integer()?
            }
            
            Some('\'') => {
                self.tokenize_char_literal()?
            }
            
            Some('"') => {
                self.tokenize_string_literal()?
            }
            
            // Single-character tokens and operators
            Some('+') => {
                self.advance();
                if self.current_char() == Some('+') {
                    self.advance();
                    TokenType::PlusPlus
                } else if self.current_char() == Some('=') {
                    self.advance();
                    TokenType::PlusEqual
                } else {
                    TokenType::Plus
                }
            }
            
            Some('-') => {
                self.advance();
                if self.current_char() == Some('-') {
                    self.advance();
                    TokenType::MinusMinus
                } else if self.current_char() == Some('=') {
                    self.advance();
                    TokenType::MinusEqual
                } else if self.current_char() == Some('>') {
                    self.advance();
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            }
            
            Some('*') => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    TokenType::StarEqual
                } else {
                    TokenType::Star
                }
            }
            
            Some('/') => {
                if self.peek_char(1) == Some('/') {
                    self.tokenize_line_comment()
                } else if self.peek_char(1) == Some('*') {
                    self.tokenize_block_comment()?
                } else {
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        TokenType::SlashEqual
                    } else {
                        TokenType::Slash
                    }
                }
            }
            
            Some('%') => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    TokenType::PercentEqual
                } else {
                    TokenType::Percent
                }
            }
            
            Some('&') => {
                self.advance();
                if self.current_char() == Some('&') {
                    self.advance();
                    TokenType::AmpersandAmpersand
                } else if self.current_char() == Some('=') {
                    self.advance();
                    TokenType::AmpersandEqual
                } else {
                    TokenType::Ampersand
                }
            }
            
            Some('|') => {
                self.advance();
                if self.current_char() == Some('|') {
                    self.advance();
                    TokenType::PipePipe
                } else if self.current_char() == Some('=') {
                    self.advance();
                    TokenType::PipeEqual
                } else {
                    TokenType::Pipe
                }
            }
            
            Some('^') => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    TokenType::CaretEqual
                } else {
                    TokenType::Caret
                }
            }
            
            Some('~') => { self.advance(); TokenType::Tilde }
            Some('!') => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }
            }
            
            Some('=') => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            
            Some('<') => {
                self.advance();
                if self.current_char() == Some('<') {
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        TokenType::LeftShiftEqual
                    } else {
                        TokenType::LeftShift
                    }
                } else if self.current_char() == Some('=') {
                    self.advance();
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            
            Some('>') => {
                self.advance();
                if self.current_char() == Some('>') {
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        TokenType::RightShiftEqual
                    } else {
                        TokenType::RightShift
                    }
                } else if self.current_char() == Some('=') {
                    self.advance();
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            
            Some('?') => { self.advance(); TokenType::Question }
            Some(':') => { self.advance(); TokenType::Colon }
            Some('(') => { self.advance(); TokenType::LeftParen }
            Some(')') => { self.advance(); TokenType::RightParen }
            Some('{') => { self.advance(); TokenType::LeftBrace }
            Some('}') => { self.advance(); TokenType::RightBrace }
            Some('[') => { self.advance(); TokenType::LeftBracket }
            Some(']') => { self.advance(); TokenType::RightBracket }
            Some(';') => { self.advance(); TokenType::Semicolon }
            Some(',') => { self.advance(); TokenType::Comma }
            Some('.') => { self.advance(); TokenType::Dot }
            
            Some(ch) => {
                return Err(CompilerError::lexer_error(
                    format!("Unexpected character: {}", ch),
                    self.current_location(),
                ));
            }
        };
        
        let end_location = self.current_location();
        let span = SourceSpan::new(start_location, end_location);
        
        Ok(Token::new(token_type, span))
    }
    
    /// Tokenize entire input into a vector of tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>, CompilerError> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.token_type, TokenType::EndOfFile);
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("int main void return if else");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 7); // 6 keywords + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Int));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(_)));
        assert!(matches!(tokens[2].token_type, TokenType::Void));
        assert!(matches!(tokens[3].token_type, TokenType::Return));
        assert!(matches!(tokens[4].token_type, TokenType::If));
        assert!(matches!(tokens[5].token_type, TokenType::Else));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / % == != <= >= && || ++ --");
        let tokens = lexer.tokenize().unwrap();
        
        let expected = vec![
            TokenType::Plus, TokenType::Minus, TokenType::Star, TokenType::Slash,
            TokenType::Percent, TokenType::EqualEqual, TokenType::BangEqual,
            TokenType::LessEqual, TokenType::GreaterEqual, TokenType::AmpersandAmpersand,
            TokenType::PipePipe, TokenType::PlusPlus, TokenType::MinusMinus,
            TokenType::EndOfFile,
        ];
        
        for (i, expected_type) in expected.iter().enumerate() {
            assert_eq!(tokens[i].token_type, *expected_type);
        }
    }

    #[test]
    fn test_literals() {
        let mut lexer = Lexer::new("42 'a' \"hello\" 0xff");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 5); // 4 literals + EOF
        assert_eq!(tokens[0].token_type, TokenType::IntLiteral(42));
        assert_eq!(tokens[1].token_type, TokenType::CharLiteral(b'a'));
        assert_eq!(tokens[2].token_type, TokenType::StringLiteral("hello".to_string()));
        assert_eq!(tokens[3].token_type, TokenType::IntLiteral(255));
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("variable _private var123 __special");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 5); // 4 identifiers + EOF
        
        match &tokens[0].token_type {
            TokenType::Identifier(name) => assert_eq!(name, "variable"),
            _ => panic!("Expected identifier"),
        }
        
        match &tokens[1].token_type {
            TokenType::Identifier(name) => assert_eq!(name, "_private"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("// line comment\n/* block comment */");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 4); // line comment + newline + block comment + EOF
        
        match &tokens[0].token_type {
            TokenType::LineComment(comment) => assert_eq!(comment, " line comment"),
            _ => panic!("Expected line comment"),
        }
        
        assert_eq!(tokens[1].token_type, TokenType::Newline);
        
        match &tokens[2].token_type {
            TokenType::BlockComment(comment) => assert_eq!(comment, " block comment "),
            _ => panic!("Expected block comment"),
        }
    }

    #[test]
    fn test_simple_function() {
        let input = r#"
int main() {
    return 42;
}
"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        // Should have: newline, int, main, (, ), {, newline, return, 42, ;, newline, }, newline, EOF
        let expected_count = 14;
        assert_eq!(tokens.len(), expected_count);
        
        assert_eq!(tokens[0].token_type, TokenType::Newline);
        assert_eq!(tokens[1].token_type, TokenType::Int);
        
        match &tokens[2].token_type {
            TokenType::Identifier(name) => assert_eq!(name, "main"),
            _ => panic!("Expected main identifier"),
        }
        
        assert_eq!(tokens[3].token_type, TokenType::LeftParen);
        assert_eq!(tokens[4].token_type, TokenType::RightParen);
        assert_eq!(tokens[5].token_type, TokenType::LeftBrace);
        assert_eq!(tokens[6].token_type, TokenType::Newline);
        assert_eq!(tokens[7].token_type, TokenType::Return);
        assert_eq!(tokens[8].token_type, TokenType::IntLiteral(42));
        assert_eq!(tokens[9].token_type, TokenType::Semicolon);
        assert_eq!(tokens[10].token_type, TokenType::Newline);
        assert_eq!(tokens[11].token_type, TokenType::RightBrace);
        assert_eq!(tokens[12].token_type, TokenType::Newline);
        assert_eq!(tokens[13].token_type, TokenType::EndOfFile);
    }

    #[test]
    fn test_string_escapes() {
        let mut lexer = Lexer::new(r#""hello\nworld\t""#);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 2); // string + EOF
        assert_eq!(tokens[0].token_type, TokenType::StringLiteral("hello\nworld\t".to_string()));
    }

    #[test]
    fn test_char_escapes() {
        let mut lexer = Lexer::new(r"'\n' '\t' '\0' '\'' '\\'");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 6); // 5 chars + EOF
        assert_eq!(tokens[0].token_type, TokenType::CharLiteral(b'\n'));
        assert_eq!(tokens[1].token_type, TokenType::CharLiteral(b'\t'));
        assert_eq!(tokens[2].token_type, TokenType::CharLiteral(0));
        assert_eq!(tokens[3].token_type, TokenType::CharLiteral(b'\''));
        assert_eq!(tokens[4].token_type, TokenType::CharLiteral(b'\\'));
    }
}