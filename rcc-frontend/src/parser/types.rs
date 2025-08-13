//! Type parsing for C99
//! 
//! This module handles parsing of type specifiers, declarators, and type-related constructs.

use crate::ast::*;
use crate::lexer::{Token, TokenType};
use crate::parser::errors::ParseError;
use crate::parser::Parser;
use rcc_common::{CompilerError, SourceSpan};
use crate::{EnumVariant, StorageClass, StructField, Type};

impl Parser {
    /// Parse storage class specifier
    pub fn parse_storage_class(&mut self) -> StorageClass {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Auto) => { self.advance(); StorageClass::Auto }
            Some(TokenType::Static) => { self.advance(); StorageClass::Static }
            Some(TokenType::Extern) => { self.advance(); StorageClass::Extern }
            Some(TokenType::Register) => { self.advance(); StorageClass::Register }
            Some(TokenType::Typedef) => { self.advance(); StorageClass::Typedef }
            _ => StorageClass::Auto, // Default storage class
        }
    }
    
    /// Parse type specifier
    pub fn parse_type_specifier(&mut self) -> Result<Type, CompilerError> {
        let location = self.current_location();
        
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Void) => { self.advance(); Ok(Type::Void) }
            Some(TokenType::Char) => { self.advance(); Ok(Type::Char) }
            Some(TokenType::Short) => { self.advance(); Ok(Type::Short) }
            Some(TokenType::Int) => { self.advance(); Ok(Type::Int) }
            Some(TokenType::Long) => { self.advance(); Ok(Type::Long) }
            Some(TokenType::Signed) => {
                self.advance();
                // Handle "signed int", "signed char", etc.
                match self.peek().map(|t| &t.token_type) {
                    Some(TokenType::Char) => { self.advance(); Ok(Type::SignedChar) }
                    Some(TokenType::Int) | None => Ok(Type::Int), // "signed" defaults to int
                    _ => Ok(Type::Int),
                }
            }
            Some(TokenType::Unsigned) => {
                self.advance();
                match self.peek().map(|t| &t.token_type) {
                    Some(TokenType::Char) => { self.advance(); Ok(Type::UnsignedChar) }
                    Some(TokenType::Short) => { self.advance(); Ok(Type::UnsignedShort) }
                    Some(TokenType::Int) => { self.advance(); Ok(Type::UnsignedInt) }
                    Some(TokenType::Long) => { self.advance(); Ok(Type::UnsignedLong) }
                    _ => Ok(Type::UnsignedInt), // "unsigned" defaults to unsigned int
                }
            }
            Some(TokenType::Struct) => {
                self.advance();
                self.parse_struct_type()
            }
            Some(TokenType::Union) => {
                self.advance();
                self.parse_union_type()
            }
            Some(TokenType::Enum) => {
                self.advance();
                self.parse_enum_type()
            }
            Some(TokenType::Identifier(name)) => {
                // Could be a typedef name
                let name = name.clone();
                self.advance();
                Ok(Type::Typedef(name))
            }
            _ => {
                Err(ParseError::InvalidType {
                    message: "Expected type specifier".to_string(),
                    location,
                }.into())
            }
        }
    }
    
    /// Parse struct type (simplified for MVP - no bitfields)
    pub fn parse_struct_type(&mut self) -> Result<Type, CompilerError> {
        let name = if let Some(Token { token_type: TokenType::Identifier(name), .. }) = self.peek() {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        };
        
        let fields = if self.match_token(&TokenType::LeftBrace) {
            let mut fields = Vec::new();
            
            while !self.check(&TokenType::RightBrace) && !self.check(&TokenType::EndOfFile) {
                let field_type = self.parse_type_specifier()?;
                let (field_name, full_field_type) = self.parse_declarator(field_type)?;
                
                fields.push(StructField {
                    name: field_name,
                    field_type: full_field_type,
                    offset: None, // Computed during semantic analysis
                });
                
                self.expect(TokenType::Semicolon, "struct field")?;
            }
            
            self.expect(TokenType::RightBrace, "struct definition")?;
            fields
        } else {
            Vec::new() // Forward declaration
        };
        
        Ok(Type::Struct { name, fields })
    }
    
    /// Parse union type
    pub fn parse_union_type(&mut self) -> Result<Type, CompilerError> {
        // Similar to struct parsing
        let name = if let Some(Token { token_type: TokenType::Identifier(name), .. }) = self.peek() {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        };
        
        let fields = if self.match_token(&TokenType::LeftBrace) {
            let mut fields = Vec::new();
            
            while !self.check(&TokenType::RightBrace) && !self.check(&TokenType::EndOfFile) {
                let field_type = self.parse_type_specifier()?;
                let (field_name, full_field_type) = self.parse_declarator(field_type)?;
                
                fields.push(StructField {
                    name: field_name,
                    field_type: full_field_type,
                    offset: None,
                });
                
                self.expect(TokenType::Semicolon, "union field")?;
            }
            
            self.expect(TokenType::RightBrace, "union definition")?;
            fields
        } else {
            Vec::new()
        };
        
        Ok(Type::Union { name, fields })
    }
    
    /// Parse enum type
    pub fn parse_enum_type(&mut self) -> Result<Type, CompilerError> {
        let name = if let Some(Token { token_type: TokenType::Identifier(name), .. }) = self.peek() {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        };
        
        let variants = if self.match_token(&TokenType::LeftBrace) {
            let mut variants = Vec::new();
            let mut current_value = 0i64;
            
            while !self.check(&TokenType::RightBrace) && !self.check(&TokenType::EndOfFile) {
                let variant_name = if let Some(Token { token_type: TokenType::Identifier(name), .. }) = self.advance() {
                    name
                } else {
                    return Err(ParseError::InvalidType {
                        message: "Expected enum variant name".to_string(),
                        location: self.current_location(),
                    }.into());
                };
                
                let value = if self.match_token(&TokenType::Equal) {
                    // Parse constant expression for enum value
                    let expr = self.parse_primary_expression()?;
                    if let ExpressionKind::IntLiteral(val) = expr.kind {
                        current_value = val;
                        val
                    } else {
                        return Err(ParseError::InvalidExpression {
                            message: "Enum values must be integer constants".to_string(),
                            location: expr.span.start,
                        }.into());
                    }
                } else {
                    current_value
                };
                
                variants.push(EnumVariant {
                    name: variant_name,
                    value,
                });
                
                current_value += 1;
                
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
            
            self.expect(TokenType::RightBrace, "enum definition")?;
            variants
        } else {
            Vec::new()
        };
        
        Ok(Type::Enum { name, variants })
    }
    
    /// Parse declarator (handles pointers, arrays, function parameters)
    pub fn parse_declarator(&mut self, base_type: Type) -> Result<(String, Type), CompilerError> {
        // Parse pointer prefix
        let mut current_type = base_type;
        while self.match_token(&TokenType::Star) {
            current_type = Type::Pointer { target: Box::new(current_type), bank: None };
        }
        
        // Parse direct declarator
        self.parse_direct_declarator(current_type)
    }
    
    /// Parse direct declarator
    pub fn parse_direct_declarator(&mut self, base_type: Type) -> Result<(String, Type), CompilerError> {
        // Get identifier name
        let name = if let Some(Token { token_type: TokenType::Identifier(name), .. }) = self.advance() {
            name
        } else {
            return Err(ParseError::InvalidType {
                message: "Expected identifier in declarator".to_string(),
                location: self.current_location(),
            }.into());
        };
        
        // Parse suffix (arrays, function parameters)
        let mut current_type = base_type;
        
        loop {
            if self.match_token(&TokenType::LeftBracket) {
                // Array declarator
                let size = if self.check(&TokenType::RightBracket) {
                    None // Incomplete array type
                } else {
                    let size_expr = self.parse_assignment_expression()?;
                    if let ExpressionKind::IntLiteral(size) = size_expr.kind {
                        if size >= 0 {
                            Some(size as u64)
                        } else {
                            return Err(ParseError::InvalidExpression {
                                message: "Array size must be non-negative".to_string(),
                                location: size_expr.span.start,
                            }.into());
                        }
                    } else {
                        return Err(ParseError::InvalidExpression {
                            message: "Array size must be a constant expression".to_string(),
                            location: size_expr.span.start,
                        }.into());
                    }
                };
                
                self.expect(TokenType::RightBracket, "array declarator")?;
                
                current_type = Type::Array {
                    element_type: Box::new(current_type),
                    size,
                };
                
            } else if self.match_token(&TokenType::LeftParen) {
                // Function declarator
                let mut parameter_types = Vec::new();
                let mut parameter_info = Vec::new(); // Store both names and types
                
                if !self.check(&TokenType::RightParen) {
                    loop {
                        // Check for void parameter list
                        if parameter_types.is_empty() && self.check(&TokenType::Void) {
                            self.advance();
                            break;
                        }
                        
                        let param_start = self.current_location();
                        let param_type = self.parse_type_specifier()?;
                        
                        // Parse declarator for the parameter (handles pointers, arrays, etc.)
                        // Check if we have a declarator (could be *, identifier, or just type)
                        let (param_name, full_param_type) = if matches!(self.peek().map(|t| &t.token_type), 
                            Some(TokenType::Star) | Some(TokenType::Identifier(_))) {
                            let (name, full_type) = self.parse_declarator(param_type)?;
                            (Some(name), full_type)
                        } else {
                            // No declarator, just the type (e.g., in function prototypes like void func(int))
                            (None, param_type)
                        };
                        
                        let param_end = self.current_location();
                        
                        parameter_types.push(full_param_type.clone());
                        parameter_info.push((param_name, full_param_type, SourceSpan::new(param_start, param_end)));
                        
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }
                
                self.expect(TokenType::RightParen, "function declarator")?;
                
                // Store parameter info in function type (we'll extract it later)
                // For now, we store it in a thread-local for the parser
                self.last_function_params = Some(parameter_info);
                
                current_type = Type::Function {
                    return_type: Box::new(current_type),
                    parameters: parameter_types,
                    is_variadic: false, // TODO: Handle variadic functions
                };
                
            } else {
                break;
            }
        }
        
        Ok((name, current_type))
    }
    
    /// Check if the current position starts a declaration
    pub fn is_declaration_start(&self) -> bool {
        matches!(self.peek().map(|t| &t.token_type), Some(
            TokenType::Auto | TokenType::Static | TokenType::Extern | TokenType::Register |
            TokenType::Typedef |
            TokenType::Void | TokenType::Char | TokenType::Short | TokenType::Int | 
            TokenType::Long | TokenType::Signed | TokenType::Unsigned |
            TokenType::Struct | TokenType::Union | TokenType::Enum
        ))
    }
}
