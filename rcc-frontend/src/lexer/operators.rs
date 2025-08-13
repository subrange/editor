//! Operator and comment parsing for the C99 lexer
//! 
//! This module handles parsing of operators and comments.

use crate::lexer::{Lexer, TokenType};
use rcc_common::CompilerError;

impl Lexer {
    /// Tokenize line comment
    pub fn tokenize_line_comment(&mut self) -> TokenType {
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
    pub fn tokenize_block_comment(&mut self) -> Result<TokenType, CompilerError> {
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
}