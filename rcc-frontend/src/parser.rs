//! C99 Recursive Descent Parser
//! 
//! Parses C99 tokens into an Abstract Syntax Tree (AST).
//! Implements a recursive descent parser for the C99 grammar.

use crate::ast::*;
use crate::lexer::{Token, TokenType};
use rcc_common::{CompilerError, SourceLocation, SourceSpan};
use std::collections::VecDeque;

/// Parse error types specific to the parser
#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: Token,
    },
    UnexpectedEndOfFile {
        expected: String,
        location: SourceLocation,
    },
    InvalidExpression {
        message: String,
        location: SourceLocation,
    },
    InvalidType {
        message: String,
        location: SourceLocation,
    },
}

impl From<ParseError> for CompilerError {
    fn from(err: ParseError) -> Self {
        match err {
            ParseError::UnexpectedToken { expected, found } => {
                CompilerError::parse_error(
                    format!("Expected {}, found {}", expected, found.token_type),
                    found.span.start,
                )
            }
            ParseError::UnexpectedEndOfFile { expected, location } => {
                CompilerError::parse_error(
                    format!("Unexpected end of file, expected {}", expected),
                    location,
                )
            }
            ParseError::InvalidExpression { message, location } => {
                CompilerError::parse_error(message, location)
            }
            ParseError::InvalidType { message, location } => {
                CompilerError::parse_error(message, location)
            }
        }
    }
}

/// C99 Parser
pub struct Parser {
    tokens: VecDeque<Token>,
    node_id_gen: NodeIdGenerator,
    last_function_params: Option<Vec<(Option<String>, Type, SourceSpan)>>, // Temporary storage for function parameters
}

impl Parser {
    /// Create a new parser
    pub fn new(tokens: Vec<Token>) -> Self {
        // Filter out comments and newlines for parsing (keep them for IDE features later)
        let filtered_tokens: Vec<Token> = tokens
            .into_iter()
            .filter(|t| !matches!(t.token_type, 
                TokenType::LineComment(_) | 
                TokenType::BlockComment(_) | 
                TokenType::Newline
            ))
            .collect();
            
        Self {
            tokens: filtered_tokens.into(),
            node_id_gen: NodeIdGenerator::new(),
            last_function_params: None,
        }
    }
    
    /// Peek at current token without consuming
    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }
    
    /// Get current token and advance
    fn advance(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }
    
    /// Check if current token matches expected type
    fn check(&self, token_type: &TokenType) -> bool {
        if let Some(token) = self.peek() {
            std::mem::discriminant(&token.token_type) == std::mem::discriminant(token_type)
        } else {
            matches!(token_type, TokenType::EndOfFile)
        }
    }
    
    /// Consume token if it matches expected type
    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    /// Expect and consume a specific token type
    fn expect(&mut self, token_type: TokenType, context: &str) -> Result<Token, ParseError> {
        if let Some(token) = self.advance() {
            if std::mem::discriminant(&token.token_type) == std::mem::discriminant(&token_type) {
                Ok(token)
            } else {
                Err(ParseError::UnexpectedToken {
                    expected: format!("{} in {}", token_type, context),
                    found: token,
                })
            }
        } else {
            let location = SourceLocation::new_simple(0, 0); // TODO: Better EOF location tracking
            Err(ParseError::UnexpectedEndOfFile {
                expected: format!("{} in {}", token_type, context),
                location,
            })
        }
    }
    
    /// Get current location for error reporting
    fn current_location(&self) -> SourceLocation {
        if let Some(token) = self.peek() {
            token.span.start.clone()
        } else {
            SourceLocation::new_simple(0, 0) // TODO: Track actual EOF location
        }
    }
    
    /// Parse a complete translation unit
    pub fn parse_translation_unit(&mut self) -> Result<TranslationUnit, CompilerError> {
        let start_location = self.current_location();
        let mut items = Vec::new();
        
        // Parse all top-level items until EOF
        while !self.check(&TokenType::EndOfFile) {
            items.push(self.parse_top_level_item()?);
        }
        
        let end_location = self.current_location();
        
        Ok(TranslationUnit {
            node_id: self.node_id_gen.next(),
            items,
            span: SourceSpan::new(start_location, end_location),
        })
    }
    
    /// Parse a top-level item (function definition or declaration)
    fn parse_top_level_item(&mut self) -> Result<TopLevelItem, CompilerError> {
        // For simplicity in MVP, we'll distinguish function definitions from declarations
        // by looking ahead for a function body (opening brace after parameter list)
        
        // Parse declaration specifiers (storage class, type)
        let storage_class = self.parse_storage_class();
        let base_type = self.parse_type_specifier()?;
        
        // Parse declarator (name and type modifications like pointers, arrays, functions)
        let (name, full_type) = self.parse_declarator(base_type)?;
        
        // Check if this is a function definition (has a body)
        if let Type::Function { .. } = full_type {
            if self.check(&TokenType::LeftBrace) {
                return Ok(TopLevelItem::Function(self.parse_function_definition(
                    name, full_type, storage_class
                )?));
            }
        }
        
        // Otherwise it's a declaration
        let declaration = self.parse_declaration_with_type(name, full_type, storage_class)?;
        Ok(TopLevelItem::Declaration(declaration))
    }
    
    /// Parse storage class specifier
    fn parse_storage_class(&mut self) -> StorageClass {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Auto) => { self.advance(); StorageClass::Auto }
            Some(TokenType::Static) => { self.advance(); StorageClass::Static }
            Some(TokenType::Extern) => { self.advance(); StorageClass::Extern }
            Some(TokenType::Register) => { self.advance(); StorageClass::Register }
            _ => StorageClass::Auto, // Default storage class
        }
    }
    
    /// Parse type specifier
    fn parse_type_specifier(&mut self) -> Result<Type, CompilerError> {
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
    fn parse_struct_type(&mut self) -> Result<Type, CompilerError> {
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
    fn parse_union_type(&mut self) -> Result<Type, CompilerError> {
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
    fn parse_enum_type(&mut self) -> Result<Type, CompilerError> {
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
    fn parse_declarator(&mut self, base_type: Type) -> Result<(String, Type), CompilerError> {
        // Parse pointer prefix
        let mut current_type = base_type;
        while self.match_token(&TokenType::Star) {
            current_type = Type::Pointer(Box::new(current_type));
        }
        
        // Parse direct declarator
        self.parse_direct_declarator(current_type)
    }
    
    /// Parse direct declarator
    fn parse_direct_declarator(&mut self, base_type: Type) -> Result<(String, Type), CompilerError> {
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
                        
                        // Parameter name is optional in function prototypes
                        let (param_name, full_param_type) = if matches!(self.peek().map(|t| &t.token_type), Some(TokenType::Identifier(_))) {
                            let (name, full_type) = self.parse_declarator(param_type)?;
                            (Some(name), full_type)
                        } else {
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
    
    /// Parse function definition
    fn parse_function_definition(
        &mut self,
        name: String,
        func_type: Type,
        storage_class: StorageClass,
    ) -> Result<FunctionDefinition, CompilerError> {
        let start_location = self.current_location();
        
        // Extract function type information
        let (return_type, _param_types) = match func_type {
            Type::Function { return_type, parameters, .. } => (*return_type, parameters),
            _ => return Err(ParseError::InvalidType {
                message: "Expected function type".to_string(),
                location: start_location,
            }.into()),
        };
        
        // Use the stored parameter information if available
        let parameters = if let Some(param_info) = self.last_function_params.take() {
            param_info.into_iter().map(|(name, param_type, span)| Parameter {
                node_id: self.node_id_gen.next(),
                name,
                param_type,
                span,
                symbol_id: None,
            }).collect()
        } else {
            Vec::new()
        };
        
        // Parse function body
        let body = self.parse_compound_statement()?;
        
        let end_location = self.current_location();
        
        Ok(FunctionDefinition {
            node_id: self.node_id_gen.next(),
            name,
            return_type,
            parameters,
            body,
            storage_class,
            span: SourceSpan::new(start_location, end_location),
            symbol_id: None,
        })
    }
    
    /// Parse declaration with known type information
    fn parse_declaration_with_type(
        &mut self,
        name: String,
        decl_type: Type,
        storage_class: StorageClass,
    ) -> Result<Declaration, CompilerError> {
        let start_location = self.current_location();
        
        // Parse optional initializer
        let initializer = if self.match_token(&TokenType::Equal) {
            Some(self.parse_initializer()?)
        } else {
            None
        };
        
        self.expect(TokenType::Semicolon, "declaration")?;
        
        let end_location = self.current_location();
        
        Ok(Declaration {
            node_id: self.node_id_gen.next(),
            name,
            decl_type,
            storage_class,
            initializer,
            span: SourceSpan::new(start_location, end_location),
            symbol_id: None,
        })
    }
    
    /// Parse initializer
    fn parse_initializer(&mut self) -> Result<Initializer, CompilerError> {
        let start_location = self.current_location();
        
        let kind = if self.match_token(&TokenType::LeftBrace) {
            // Initializer list
            let mut initializers = Vec::new();
            
            if !self.check(&TokenType::RightBrace) {
                loop {
                    initializers.push(self.parse_initializer()?);
                    
                    if !self.match_token(&TokenType::Comma) {
                        break;
                    }
                    
                    // Allow trailing comma
                    if self.check(&TokenType::RightBrace) {
                        break;
                    }
                }
            }
            
            self.expect(TokenType::RightBrace, "initializer list")?;
            InitializerKind::List(initializers)
        } else {
            // Expression initializer
            let expr = self.parse_assignment_expression()?;
            InitializerKind::Expression(expr)
        };
        
        let end_location = self.current_location();
        
        Ok(Initializer {
            node_id: self.node_id_gen.next(),
            kind,
            span: SourceSpan::new(start_location, end_location),
        })
    }
    
    /// Parse compound statement (block)
    fn parse_compound_statement(&mut self) -> Result<Statement, CompilerError> {
        let start_location = self.current_location();
        
        self.expect(TokenType::LeftBrace, "compound statement")?;
        
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.check(&TokenType::EndOfFile) {
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenType::RightBrace, "compound statement")?;
        
        let end_location = self.current_location();
        
        Ok(Statement {
            node_id: self.node_id_gen.next(),
            kind: StatementKind::Compound(statements),
            span: SourceSpan::new(start_location, end_location),
        })
    }
    
    /// Parse statement
    fn parse_statement(&mut self) -> Result<Statement, CompilerError> {
        let start_location = self.current_location();
        
        let kind = match self.peek().map(|t| &t.token_type) {
            Some(TokenType::LeftBrace) => {
                return self.parse_compound_statement();
            }
            Some(TokenType::If) => {
                self.advance();
                self.parse_if_statement()?
            }
            Some(TokenType::While) => {
                self.advance();
                self.parse_while_statement()?
            }
            Some(TokenType::For) => {
                self.advance();
                self.parse_for_statement()?
            }
            Some(TokenType::Do) => {
                self.advance();
                self.parse_do_while_statement()?
            }
            Some(TokenType::Return) => {
                self.advance();
                self.parse_return_statement()?
            }
            Some(TokenType::Break) => {
                self.advance();
                self.expect(TokenType::Semicolon, "break statement")?;
                StatementKind::Break
            }
            Some(TokenType::Continue) => {
                self.advance();
                self.expect(TokenType::Semicolon, "continue statement")?;
                StatementKind::Continue
            }
            Some(TokenType::Semicolon) => {
                self.advance();
                StatementKind::Empty
            }
            // Check for declaration vs expression statement
            _ => {
                // This is a simplified check - a full parser would need more lookahead
                if self.is_declaration_start() {
                    self.parse_declaration_statement()?
                } else {
                    self.parse_expression_statement()?
                }
            }
        };
        
        let end_location = self.current_location();
        
        Ok(Statement {
            node_id: self.node_id_gen.next(),
            kind,
            span: SourceSpan::new(start_location, end_location),
        })
    }
    
    /// Check if the current position starts a declaration
    fn is_declaration_start(&self) -> bool {
        matches!(self.peek().map(|t| &t.token_type), Some(
            TokenType::Auto | TokenType::Static | TokenType::Extern | TokenType::Register |
            TokenType::Void | TokenType::Char | TokenType::Short | TokenType::Int | 
            TokenType::Long | TokenType::Signed | TokenType::Unsigned |
            TokenType::Struct | TokenType::Union | TokenType::Enum
        ))
    }
    
    /// Parse declaration statement
    fn parse_declaration_statement(&mut self) -> Result<StatementKind, CompilerError> {
        let storage_class = self.parse_storage_class();
        let base_type = self.parse_type_specifier()?;
        
        let mut declarations = Vec::new();
        
        // Parse first declarator
        let (name, full_type) = self.parse_declarator(base_type.clone())?;
        let initializer = if self.match_token(&TokenType::Equal) {
            Some(self.parse_initializer()?)
        } else {
            None
        };
        
        declarations.push(Declaration {
            node_id: self.node_id_gen.next(),
            name,
            decl_type: full_type,
            storage_class,
            initializer,
            span: SourceSpan::new(self.current_location(), self.current_location()),
            symbol_id: None,
        });
        
        // Parse additional declarators (comma-separated)
        while self.match_token(&TokenType::Comma) {
            let (name, full_type) = self.parse_declarator(base_type.clone())?;
            let initializer = if self.match_token(&TokenType::Equal) {
                Some(self.parse_initializer()?)
            } else {
                None
            };
            
            declarations.push(Declaration {
                node_id: self.node_id_gen.next(),
                name,
                decl_type: full_type,
                storage_class,
                initializer,
                span: SourceSpan::new(self.current_location(), self.current_location()),
                symbol_id: None,
            });
        }
        
        self.expect(TokenType::Semicolon, "declaration")?;
        
        Ok(StatementKind::Declaration { declarations })
    }
    
    /// Parse expression statement
    fn parse_expression_statement(&mut self) -> Result<StatementKind, CompilerError> {
        let expr = self.parse_expression()?;
        self.expect(TokenType::Semicolon, "expression statement")?;
        Ok(StatementKind::Expression(expr))
    }
    
    /// Parse if statement
    fn parse_if_statement(&mut self) -> Result<StatementKind, CompilerError> {
        self.expect(TokenType::LeftParen, "if statement")?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RightParen, "if statement")?;
        
        let then_stmt = Box::new(self.parse_statement()?);
        
        let else_stmt = if self.match_token(&TokenType::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        
        Ok(StatementKind::If { condition, then_stmt, else_stmt })
    }
    
    /// Parse while statement
    fn parse_while_statement(&mut self) -> Result<StatementKind, CompilerError> {
        self.expect(TokenType::LeftParen, "while statement")?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RightParen, "while statement")?;
        
        let body = Box::new(self.parse_statement()?);
        
        Ok(StatementKind::While { condition, body })
    }
    
    /// Parse for statement
    fn parse_for_statement(&mut self) -> Result<StatementKind, CompilerError> {
        self.expect(TokenType::LeftParen, "for statement")?;
        
        // Parse init (can be declaration or expression)
        let init = if self.check(&TokenType::Semicolon) {
            None
        } else if self.is_declaration_start() {
            Some(Box::new(Statement {
                node_id: self.node_id_gen.next(),
                kind: self.parse_declaration_statement()?,
                span: SourceSpan::new(self.current_location(), self.current_location()),
            }))
        } else {
            let expr = self.parse_expression()?;
            self.expect(TokenType::Semicolon, "for statement init")?;
            Some(Box::new(Statement {
                node_id: self.node_id_gen.next(),
                kind: StatementKind::Expression(expr),
                span: SourceSpan::new(self.current_location(), self.current_location()),
            }))
        };
        
        if init.is_none() {
            self.expect(TokenType::Semicolon, "for statement init")?;
        }
        
        // Parse condition
        let condition = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(TokenType::Semicolon, "for statement condition")?;
        
        // Parse update
        let update = if self.check(&TokenType::RightParen) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(TokenType::RightParen, "for statement")?;
        
        let body = Box::new(self.parse_statement()?);
        
        Ok(StatementKind::For { init, condition, update, body })
    }
    
    /// Parse do-while statement
    fn parse_do_while_statement(&mut self) -> Result<StatementKind, CompilerError> {
        let body = Box::new(self.parse_statement()?);
        
        self.expect(TokenType::While, "do-while statement")?;
        self.expect(TokenType::LeftParen, "do-while statement")?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RightParen, "do-while statement")?;
        self.expect(TokenType::Semicolon, "do-while statement")?;
        
        Ok(StatementKind::DoWhile { body, condition })
    }
    
    /// Parse return statement
    fn parse_return_statement(&mut self) -> Result<StatementKind, CompilerError> {
        let value = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        
        self.expect(TokenType::Semicolon, "return statement")?;
        Ok(StatementKind::Return(value))
    }
    
    /// Parse expression (top level)
    fn parse_expression(&mut self) -> Result<Expression, CompilerError> {
        self.parse_assignment_expression()
    }
    
    /// Parse assignment expression
    fn parse_assignment_expression(&mut self) -> Result<Expression, CompilerError> {
        let left = self.parse_conditional_expression()?;
        
        // Check for assignment operators
        if let Some(op) = self.parse_assignment_operator() {
            let right = self.parse_assignment_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            return Ok(Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            });
        }
        
        Ok(left)
    }
    
    /// Parse assignment operator
    fn parse_assignment_operator(&mut self) -> Option<BinaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Equal) => { self.advance(); Some(BinaryOp::Assign) }
            Some(TokenType::PlusEqual) => { self.advance(); Some(BinaryOp::AddAssign) }
            Some(TokenType::MinusEqual) => { self.advance(); Some(BinaryOp::SubAssign) }
            Some(TokenType::StarEqual) => { self.advance(); Some(BinaryOp::MulAssign) }
            Some(TokenType::SlashEqual) => { self.advance(); Some(BinaryOp::DivAssign) }
            Some(TokenType::PercentEqual) => { self.advance(); Some(BinaryOp::ModAssign) }
            _ => None,
        }
    }
    
    /// Parse conditional expression (ternary operator)
    fn parse_conditional_expression(&mut self) -> Result<Expression, CompilerError> {
        let condition = self.parse_logical_or_expression()?;
        
        if self.match_token(&TokenType::Question) {
            let then_expr = self.parse_expression()?;
            self.expect(TokenType::Colon, "conditional expression")?;
            let else_expr = self.parse_conditional_expression()?;
            
            let span = SourceSpan::new(condition.span.start.clone(), else_expr.span.end.clone());
            
            Ok(Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Conditional {
                    condition: Box::new(condition),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                },
                span,
                expr_type: None,
            })
        } else {
            Ok(condition)
        }
    }
    
    /// Parse logical OR expression
    fn parse_logical_or_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_logical_and_expression()?;
        
        while self.match_token(&TokenType::PipePipe) {
            let right = self.parse_logical_and_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op: BinaryOp::LogicalOr,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse logical AND expression
    fn parse_logical_and_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_bitwise_or_expression()?;
        
        while self.match_token(&TokenType::AmpersandAmpersand) {
            let right = self.parse_bitwise_or_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op: BinaryOp::LogicalAnd,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse bitwise OR expression
    fn parse_bitwise_or_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_bitwise_xor_expression()?;
        
        while self.match_token(&TokenType::Pipe) {
            let right = self.parse_bitwise_xor_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op: BinaryOp::BitOr,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse bitwise XOR expression
    fn parse_bitwise_xor_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_bitwise_and_expression()?;
        
        while self.match_token(&TokenType::Caret) {
            let right = self.parse_bitwise_and_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op: BinaryOp::BitXor,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse bitwise AND expression
    fn parse_bitwise_and_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_equality_expression()?;
        
        while self.match_token(&TokenType::Ampersand) {
            let right = self.parse_equality_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op: BinaryOp::BitAnd,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse equality expression
    fn parse_equality_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_relational_expression()?;
        
        while let Some(op) = self.parse_equality_operator() {
            let right = self.parse_relational_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse equality operator
    fn parse_equality_operator(&mut self) -> Option<BinaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::EqualEqual) => { self.advance(); Some(BinaryOp::Equal) }
            Some(TokenType::BangEqual) => { self.advance(); Some(BinaryOp::NotEqual) }
            _ => None,
        }
    }
    
    /// Parse relational expression
    fn parse_relational_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_shift_expression()?;
        
        while let Some(op) = self.parse_relational_operator() {
            let right = self.parse_shift_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse relational operator
    fn parse_relational_operator(&mut self) -> Option<BinaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Less) => { self.advance(); Some(BinaryOp::Less) }
            Some(TokenType::Greater) => { self.advance(); Some(BinaryOp::Greater) }
            Some(TokenType::LessEqual) => { self.advance(); Some(BinaryOp::LessEqual) }
            Some(TokenType::GreaterEqual) => { self.advance(); Some(BinaryOp::GreaterEqual) }
            _ => None,
        }
    }
    
    /// Parse shift expression
    fn parse_shift_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_additive_expression()?;
        
        while let Some(op) = self.parse_shift_operator() {
            let right = self.parse_additive_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse shift operator
    fn parse_shift_operator(&mut self) -> Option<BinaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::LeftShift) => { self.advance(); Some(BinaryOp::LeftShift) }
            Some(TokenType::RightShift) => { self.advance(); Some(BinaryOp::RightShift) }
            _ => None,
        }
    }
    
    /// Parse additive expression
    fn parse_additive_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_multiplicative_expression()?;
        
        while let Some(op) = self.parse_additive_operator() {
            let right = self.parse_multiplicative_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse additive operator
    fn parse_additive_operator(&mut self) -> Option<BinaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Plus) => { self.advance(); Some(BinaryOp::Add) }
            Some(TokenType::Minus) => { self.advance(); Some(BinaryOp::Sub) }
            _ => None,
        }
    }
    
    /// Parse multiplicative expression
    fn parse_multiplicative_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut left = self.parse_unary_expression()?;
        
        while let Some(op) = self.parse_multiplicative_operator() {
            let right = self.parse_unary_expression()?;
            let span = SourceSpan::new(left.span.start.clone(), right.span.end.clone());
            
            left = Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                expr_type: None,
            };
        }
        
        Ok(left)
    }
    
    /// Parse multiplicative operator
    fn parse_multiplicative_operator(&mut self) -> Option<BinaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Star) => { self.advance(); Some(BinaryOp::Mul) }
            Some(TokenType::Slash) => { self.advance(); Some(BinaryOp::Div) }
            Some(TokenType::Percent) => { self.advance(); Some(BinaryOp::Mod) }
            _ => None,
        }
    }
    
    /// Parse unary expression
    fn parse_unary_expression(&mut self) -> Result<Expression, CompilerError> {
        if let Some(op) = self.parse_unary_operator() {
            let operand = self.parse_unary_expression()?;
            let span = SourceSpan::new(self.current_location(), operand.span.end.clone());
            
            Ok(Expression {
                node_id: self.node_id_gen.next(),
                kind: ExpressionKind::Unary {
                    op,
                    operand: Box::new(operand),
                },
                span,
                expr_type: None,
            })
        } else {
            self.parse_postfix_expression()
        }
    }
    
    /// Parse unary operator
    fn parse_unary_operator(&mut self) -> Option<UnaryOp> {
        match self.peek().map(|t| &t.token_type) {
            Some(TokenType::Plus) => { self.advance(); Some(UnaryOp::Plus) }
            Some(TokenType::Minus) => { self.advance(); Some(UnaryOp::Minus) }
            Some(TokenType::Bang) => { self.advance(); Some(UnaryOp::LogicalNot) }
            Some(TokenType::Tilde) => { self.advance(); Some(UnaryOp::BitNot) }
            Some(TokenType::Star) => { self.advance(); Some(UnaryOp::Dereference) }
            Some(TokenType::Ampersand) => { self.advance(); Some(UnaryOp::AddressOf) }
            Some(TokenType::PlusPlus) => { self.advance(); Some(UnaryOp::PreIncrement) }
            Some(TokenType::MinusMinus) => { self.advance(); Some(UnaryOp::PreDecrement) }
            Some(TokenType::Sizeof) => { self.advance(); Some(UnaryOp::Sizeof) }
            _ => None,
        }
    }
    
    /// Parse postfix expression
    fn parse_postfix_expression(&mut self) -> Result<Expression, CompilerError> {
        let mut expr = self.parse_primary_expression()?;
        
        loop {
            match self.peek().map(|t| &t.token_type) {
                Some(TokenType::LeftBracket) => {
                    // Array indexing
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(TokenType::RightBracket, "array index")?;
                    
                    let span = SourceSpan::new(expr.span.start.clone(), self.current_location());
                    
                    expr = Expression {
                        node_id: self.node_id_gen.next(),
                        kind: ExpressionKind::Binary {
                            op: BinaryOp::Index,
                            left: Box::new(expr),
                            right: Box::new(index),
                        },
                        span,
                        expr_type: None,
                    };
                }
                Some(TokenType::LeftParen) => {
                    // Function call
                    self.advance();
                    let mut arguments = Vec::new();
                    
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            arguments.push(self.parse_assignment_expression()?);
                            
                            if !self.match_token(&TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    
                    self.expect(TokenType::RightParen, "function call")?;
                    
                    let span = SourceSpan::new(expr.span.start.clone(), self.current_location());
                    
                    expr = Expression {
                        node_id: self.node_id_gen.next(),
                        kind: ExpressionKind::Call {
                            function: Box::new(expr),
                            arguments,
                        },
                        span,
                        expr_type: None,
                    };
                }
                Some(TokenType::Dot) => {
                    // Member access
                    self.advance();
                    let member = if let Some(Token { token_type: TokenType::Identifier(name), .. }) = self.advance() {
                        name
                    } else {
                        return Err(ParseError::InvalidExpression {
                            message: "Expected member name after '.'".to_string(),
                            location: self.current_location(),
                        }.into());
                    };
                    
                    let span = SourceSpan::new(expr.span.start.clone(), self.current_location());
                    
                    expr = Expression {
                        node_id: self.node_id_gen.next(),
                        kind: ExpressionKind::Member {
                            object: Box::new(expr),
                            member,
                            is_pointer: false,
                        },
                        span,
                        expr_type: None,
                    };
                }
                Some(TokenType::Arrow) => {
                    // Pointer member access
                    self.advance();
                    let member = if let Some(Token { token_type: TokenType::Identifier(name), .. }) = self.advance() {
                        name
                    } else {
                        return Err(ParseError::InvalidExpression {
                            message: "Expected member name after '->'".to_string(),
                            location: self.current_location(),
                        }.into());
                    };
                    
                    let span = SourceSpan::new(expr.span.start.clone(), self.current_location());
                    
                    expr = Expression {
                        node_id: self.node_id_gen.next(),
                        kind: ExpressionKind::Member {
                            object: Box::new(expr),
                            member,
                            is_pointer: true,
                        },
                        span,
                        expr_type: None,
                    };
                }
                Some(TokenType::PlusPlus) => {
                    // Postfix increment
                    self.advance();
                    let span = SourceSpan::new(expr.span.start.clone(), self.current_location());
                    
                    expr = Expression {
                        node_id: self.node_id_gen.next(),
                        kind: ExpressionKind::Unary {
                            op: UnaryOp::PostIncrement,
                            operand: Box::new(expr),
                        },
                        span,
                        expr_type: None,
                    };
                }
                Some(TokenType::MinusMinus) => {
                    // Postfix decrement
                    self.advance();
                    let span = SourceSpan::new(expr.span.start.clone(), self.current_location());
                    
                    expr = Expression {
                        node_id: self.node_id_gen.next(),
                        kind: ExpressionKind::Unary {
                            op: UnaryOp::PostDecrement,
                            operand: Box::new(expr),
                        },
                        span,
                        expr_type: None,
                    };
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    /// Parse primary expression
    fn parse_primary_expression(&mut self) -> Result<Expression, CompilerError> {
        let start_location = self.current_location();
        
        let kind = match self.advance() {
            Some(Token { token_type: TokenType::IntLiteral(value), .. }) => {
                ExpressionKind::IntLiteral(value)
            }
            Some(Token { token_type: TokenType::CharLiteral(value), .. }) => {
                ExpressionKind::CharLiteral(value)
            }
            Some(Token { token_type: TokenType::StringLiteral(value), .. }) => {
                ExpressionKind::StringLiteral(value)
            }
            Some(Token { token_type: TokenType::Identifier(name), .. }) => {
                ExpressionKind::Identifier { name, symbol_id: None }
            }
            Some(Token { token_type: TokenType::LeftParen, .. }) => {
                // Parenthesized expression
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen, "parenthesized expression")?;
                return Ok(expr);
            }
            Some(token) => {
                return Err(ParseError::UnexpectedToken {
                    expected: "primary expression".to_string(),
                    found: token,
                }.into());
            }
            None => {
                return Err(ParseError::UnexpectedEndOfFile {
                    expected: "primary expression".to_string(),
                    location: start_location,
                }.into());
            }
        };
        
        let end_location = self.current_location();
        
        Ok(Expression {
            node_id: self.node_id_gen.next(),
            kind,
            span: SourceSpan::new(start_location, end_location),
            expr_type: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_expression_from_str(input: &str) -> Result<Expression, CompilerError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse_expression()
    }
    
    fn parse_statement_from_str(input: &str) -> Result<Statement, CompilerError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse_statement()
    }

    #[test]
    fn test_parse_integer_literal() {
        let expr = parse_expression_from_str("42").unwrap();
        match expr.kind {
            ExpressionKind::IntLiteral(value) => assert_eq!(value, 42),
            _ => panic!("Expected integer literal"),
        }
    }

    #[test]
    fn test_parse_identifier() {
        let expr = parse_expression_from_str("variable").unwrap();
        match expr.kind {
            ExpressionKind::Identifier { name, .. } => assert_eq!(name, "variable"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        let expr = parse_expression_from_str("2 + 3").unwrap();
        match expr.kind {
            ExpressionKind::Binary { op, left, right } => {
                assert_eq!(op, BinaryOp::Add);
                match (&left.kind, &right.kind) {
                    (ExpressionKind::IntLiteral(2), ExpressionKind::IntLiteral(3)) => {},
                    _ => panic!("Expected 2 + 3"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let expr = parse_expression_from_str("foo(1, 2)").unwrap();
        match expr.kind {
            ExpressionKind::Call { function, arguments } => {
                match &function.kind {
                    ExpressionKind::Identifier { name, .. } => assert_eq!(name, "foo"),
                    _ => panic!("Expected function identifier"),
                }
                assert_eq!(arguments.len(), 2);
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_parse_unary_expression() {
        let expr = parse_expression_from_str("-42").unwrap();
        match expr.kind {
            ExpressionKind::Unary { op, operand } => {
                assert_eq!(op, UnaryOp::Minus);
                match &operand.kind {
                    ExpressionKind::IntLiteral(42) => {},
                    _ => panic!("Expected -42"),
                }
            }
            _ => panic!("Expected unary expression"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let stmt = parse_statement_from_str("if (x > 0) return 1;").unwrap();
        match stmt.kind {
            StatementKind::If { condition, then_stmt, else_stmt } => {
                match condition.kind {
                    ExpressionKind::Binary { op: BinaryOp::Greater, .. } => {},
                    _ => panic!("Expected comparison condition"),
                }
                match &then_stmt.kind {
                    StatementKind::Return(Some(_)) => {},
                    _ => panic!("Expected return statement"),
                }
                assert!(else_stmt.is_none());
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_parse_compound_statement() {
        let stmt = parse_statement_from_str("{ int x = 5; return x; }").unwrap();
        match stmt.kind {
            StatementKind::Compound(statements) => {
                assert_eq!(statements.len(), 2);
                match &statements[0].kind {
                    StatementKind::Declaration { .. } => {},
                    _ => panic!("Expected declaration"),
                }
                match &statements[1].kind {
                    StatementKind::Return(_) => {},
                    _ => panic!("Expected return statement"),
                }
            }
            _ => panic!("Expected compound statement"),
        }
    }

    #[test]
    fn test_parse_simple_function() {
        let input = "int main() { return 42; }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let translation_unit = parser.parse_translation_unit().unwrap();
        assert_eq!(translation_unit.items.len(), 1);
        
        match &translation_unit.items[0] {
            TopLevelItem::Function(func) => {
                assert_eq!(func.name, "main");
                assert_eq!(func.return_type, Type::Int);
                match &func.body.kind {
                    StatementKind::Compound(statements) => {
                        assert_eq!(statements.len(), 1);
                        match &statements[0].kind {
                            StatementKind::Return(Some(expr)) => {
                                match &expr.kind {
                                    ExpressionKind::IntLiteral(42) => {},
                                    _ => panic!("Expected return 42"),
                                }
                            }
                            _ => panic!("Expected return statement"),
                        }
                    }
                    _ => panic!("Expected compound statement"),
                }
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_operator_precedence() {
        let expr = parse_expression_from_str("2 + 3 * 4").unwrap();
        
        // Should parse as 2 + (3 * 4)
        match expr.kind {
            ExpressionKind::Binary { op: BinaryOp::Add, left, right } => {
                match (&left.kind, &right.kind) {
                    (ExpressionKind::IntLiteral(2), ExpressionKind::Binary { op: BinaryOp::Mul, .. }) => {},
                    _ => panic!("Expected 2 + (3 * 4) structure"),
                }
            }
            _ => panic!("Expected binary addition"),
        }
    }
}