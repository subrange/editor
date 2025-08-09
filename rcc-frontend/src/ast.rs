//! Abstract Syntax Tree definitions for C99
//! 
//! This module defines the AST nodes that represent C99 language constructs.
//! The AST is built by the parser and used by semantic analysis and IR generation.

use rcc_common::{SourceSpan, IntType, SymbolId};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for AST nodes (useful for debugging and analysis)
pub type NodeId = u32;

/// C99 type system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    /// Void type
    Void,
    
    /// Boolean type (_Bool)
    Bool,
    
    /// Character types
    Char,
    SignedChar,
    UnsignedChar,
    
    /// Integer types
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    Long,
    UnsignedLong,
    
    /// Pointer to another type
    Pointer(Box<Type>),
    
    /// Array type with optional size
    Array {
        element_type: Box<Type>,
        size: Option<u64>,
    },
    
    /// Function type
    Function {
        return_type: Box<Type>,
        parameters: Vec<Type>,
        is_variadic: bool,
    },
    
    /// Struct type (simplified - no bitfields in MVP)
    Struct {
        name: Option<String>,
        fields: Vec<StructField>,
    },
    
    /// Union type
    Union {
        name: Option<String>,
        fields: Vec<StructField>,
    },
    
    /// Enum type
    Enum {
        name: Option<String>,
        variants: Vec<EnumVariant>,
    },
    
    /// Typedef alias
    Typedef(String),
    
    /// Placeholder for type resolution errors
    Error,
}

impl Type {
    /// Get the size of this type in bytes
    pub fn size_in_bytes(&self) -> Option<u64> {
        match self {
            Type::Void => None,
            Type::Bool | Type::Char | Type::SignedChar | Type::UnsignedChar => Some(1),
            Type::Short | Type::UnsignedShort => Some(2),
            Type::Int | Type::UnsignedInt => Some(2), // 16-bit int on Ripple
            Type::Long | Type::UnsignedLong => Some(4),
            Type::Pointer(_) => Some(2), // 16-bit pointers in MVP
            Type::Array { element_type, size: Some(count) } => {
                element_type.size_in_bytes().map(|elem_size| elem_size * count)
            }
            Type::Array { size: None, .. } => None, // Incomplete type
            Type::Function { .. } => None, // Functions don't have size
            Type::Struct { fields, .. } => {
                let mut total = 0;
                for field in fields {
                    total += field.field_type.size_in_bytes()?;
                }
                Some(total)
            }
            Type::Union { fields, .. } => {
                fields.iter()
                    .map(|f| f.field_type.size_in_bytes())
                    .flatten()
                    .max()
            }
            Type::Enum { .. } => Some(2), // Enum is like int
            Type::Typedef(_) | Type::Error => None,
        }
    }
    
    /// Get the size in 16-bit words (Ripple VM cells)
    pub fn size_in_words(&self) -> Option<u64> {
        self.size_in_bytes().map(|bytes| (bytes + 1) / 2)
    }
    
    /// Check if type is integer
    pub fn is_integer(&self) -> bool {
        matches!(self, 
            Type::Bool | Type::Char | Type::SignedChar | Type::UnsignedChar |
            Type::Short | Type::UnsignedShort | Type::Int | Type::UnsignedInt |
            Type::Long | Type::UnsignedLong | Type::Enum { .. }
        )
    }
    
    /// Check if type is signed integer
    pub fn is_signed_integer(&self) -> bool {
        matches!(self, 
            Type::Char | Type::SignedChar | Type::Short | Type::Int | Type::Long
        )
    }
    
    /// Check if type is pointer
    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_) | Type::Array { .. })
    }
    
    /// Get pointer target type
    pub fn pointer_target(&self) -> Option<&Type> {
        match self {
            Type::Pointer(target) => Some(target),
            Type::Array { element_type, .. } => Some(element_type),
            _ => None,
        }
    }
    
    /// Check if this type is compatible with another for assignment
    pub fn is_assignable_from(&self, other: &Type) -> bool {
        // Simplified compatibility rules for MVP
        match (self, other) {
            // Exact match
            (a, b) if a == b => true,
            
            // Integer conversions
            (a, b) if a.is_integer() && b.is_integer() => true,
            
            // Pointer conversions
            (Type::Pointer(a), Type::Pointer(b)) => {
                // void* is compatible with any pointer
                matches!(a.as_ref(), Type::Void) || matches!(b.as_ref(), Type::Void) || a == b
            }
            
            // Array to pointer decay
            (Type::Pointer(target), Type::Array { element_type, .. }) => target == element_type,
            
            _ => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Bool => write!(f, "_Bool"),
            Type::Char => write!(f, "char"),
            Type::SignedChar => write!(f, "signed char"),
            Type::UnsignedChar => write!(f, "unsigned char"),
            Type::Short => write!(f, "short"),
            Type::UnsignedShort => write!(f, "unsigned short"),
            Type::Int => write!(f, "int"),
            Type::UnsignedInt => write!(f, "unsigned int"),
            Type::Long => write!(f, "long"),
            Type::UnsignedLong => write!(f, "unsigned long"),
            Type::Pointer(target) => write!(f, "{}*", target),
            Type::Array { element_type, size: Some(n) } => write!(f, "{}[{}]", element_type, n),
            Type::Array { element_type, size: None } => write!(f, "{}[]", element_type),
            Type::Function { return_type, parameters, is_variadic } => {
                write!(f, "{} (", return_type)?;
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                if *is_variadic { write!(f, ", ...")?; }
                write!(f, ")")
            }
            Type::Struct { name: Some(name), .. } => write!(f, "struct {}", name),
            Type::Struct { name: None, .. } => write!(f, "struct <anonymous>"),
            Type::Union { name: Some(name), .. } => write!(f, "union {}", name),
            Type::Union { name: None, .. } => write!(f, "union <anonymous>"),
            Type::Enum { name: Some(name), .. } => write!(f, "enum {}", name),
            Type::Enum { name: None, .. } => write!(f, "enum <anonymous>"),
            Type::Typedef(name) => write!(f, "{}", name),
            Type::Error => write!(f, "<error>"),
        }
    }
}

/// Struct/union field
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
    pub offset: Option<u64>, // Computed during semantic analysis
}

/// Enum variant
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub value: i64,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    
    // Bitwise
    BitAnd, BitOr, BitXor, LeftShift, RightShift,
    
    // Logical
    LogicalAnd, LogicalOr,
    
    // Comparison
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    
    // Assignment
    Assign,
    AddAssign, SubAssign, MulAssign, DivAssign, ModAssign,
    BitAndAssign, BitOrAssign, BitXorAssign, LeftShiftAssign, RightShiftAssign,
    
    // Array/pointer access
    Index,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::LeftShift => "<<",
            BinaryOp::RightShift => ">>",
            BinaryOp::LogicalAnd => "&&",
            BinaryOp::LogicalOr => "||",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Less => "<",
            BinaryOp::Greater => ">",
            BinaryOp::LessEqual => "<=",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::Assign => "=",
            BinaryOp::AddAssign => "+=",
            BinaryOp::SubAssign => "-=",
            BinaryOp::MulAssign => "*=",
            BinaryOp::DivAssign => "/=",
            BinaryOp::ModAssign => "%=",
            BinaryOp::BitAndAssign => "&=",
            BinaryOp::BitOrAssign => "|=",
            BinaryOp::BitXorAssign => "^=",
            BinaryOp::LeftShiftAssign => "<<=",
            BinaryOp::RightShiftAssign => ">>=",
            BinaryOp::Index => "[]",
        };
        write!(f, "{}", op_str)
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    // Arithmetic
    Plus, Minus,
    
    // Bitwise
    BitNot,
    
    // Logical
    LogicalNot,
    
    // Pointer/address
    Dereference, AddressOf,
    
    // Pre/post increment/decrement
    PreIncrement, PostIncrement,
    PreDecrement, PostDecrement,
    
    // Sizeof
    Sizeof,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self {
            UnaryOp::Plus => "+",
            UnaryOp::Minus => "-",
            UnaryOp::BitNot => "~",
            UnaryOp::LogicalNot => "!",
            UnaryOp::Dereference => "*",
            UnaryOp::AddressOf => "&",
            UnaryOp::PreIncrement => "++",
            UnaryOp::PostIncrement => "++",
            UnaryOp::PreDecrement => "--",
            UnaryOp::PostDecrement => "--",
            UnaryOp::Sizeof => "sizeof",
        };
        write!(f, "{}", op_str)
    }
}

/// AST Expression nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expression {
    pub node_id: NodeId,
    pub kind: ExpressionKind,
    pub span: SourceSpan,
    pub expr_type: Option<Type>, // Filled during semantic analysis
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpressionKind {
    /// Integer literal
    IntLiteral(i64),
    
    /// Character literal
    CharLiteral(u8),
    
    /// String literal
    StringLiteral(String),
    
    /// Identifier reference
    Identifier {
        name: String,
        symbol_id: Option<SymbolId>, // Filled during semantic analysis
    },
    
    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    
    /// Unary operation
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
    
    /// Function call
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    
    /// Array/struct member access
    Member {
        object: Box<Expression>,
        member: String,
        is_pointer: bool, // true for ->, false for .
    },
    
    /// Ternary conditional operator (condition ? then_expr : else_expr)
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
    
    /// Type cast
    Cast {
        target_type: Type,
        operand: Box<Expression>,
    },
    
    /// Sizeof expression
    SizeofExpr(Box<Expression>),
    
    /// Sizeof type
    SizeofType(Type),
    
    /// Compound literal (C99)
    CompoundLiteral {
        type_name: Type,
        initializer: Box<Initializer>,
    },
}

/// Initializer for variables, arrays, structs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Initializer {
    pub node_id: NodeId,
    pub kind: InitializerKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InitializerKind {
    /// Single expression
    Expression(Expression),
    
    /// Initializer list (for arrays, structs)
    List(Vec<Initializer>),
    
    /// Designated initializer (C99): .field = value or [index] = value
    Designated {
        designator: Designator,
        initializer: Box<Initializer>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Designator {
    /// Array index: [index]
    Index(Expression),
    
    /// Struct member: .member
    Member(String),
}

/// AST Statement nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statement {
    pub node_id: NodeId,
    pub kind: StatementKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatementKind {
    /// Expression statement
    Expression(Expression),
    
    /// Compound statement (block)
    Compound(Vec<Statement>),
    
    /// Variable declaration
    Declaration {
        declarations: Vec<Declaration>,
    },
    
    /// If statement
    If {
        condition: Expression,
        then_stmt: Box<Statement>,
        else_stmt: Option<Box<Statement>>,
    },
    
    /// While loop
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    
    /// For loop
    For {
        init: Option<Box<Statement>>, // Can be declaration or expression
        condition: Option<Expression>,
        update: Option<Expression>,
        body: Box<Statement>,
    },
    
    /// Do-while loop
    DoWhile {
        body: Box<Statement>,
        condition: Expression,
    },
    
    /// Switch statement
    Switch {
        expression: Expression,
        body: Box<Statement>,
    },
    
    /// Case label
    Case {
        value: Expression,
        statement: Box<Statement>,
    },
    
    /// Default case
    Default {
        statement: Box<Statement>,
    },
    
    /// Break statement
    Break,
    
    /// Continue statement
    Continue,
    
    /// Return statement
    Return(Option<Expression>),
    
    /// Goto statement
    Goto(String),
    
    /// Label statement
    Label {
        name: String,
        statement: Box<Statement>,
    },
    
    /// Empty statement (just semicolon)
    Empty,
}

/// Variable/function declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Declaration {
    pub node_id: NodeId,
    pub name: String,
    pub decl_type: Type,
    pub storage_class: StorageClass,
    pub initializer: Option<Initializer>,
    pub span: SourceSpan,
    pub symbol_id: Option<SymbolId>, // Filled during semantic analysis
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageClass {
    Auto,
    Static,
    Extern,
    Register,
    Typedef,
}

impl fmt::Display for StorageClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let class_str = match self {
            StorageClass::Auto => "auto",
            StorageClass::Static => "static",
            StorageClass::Extern => "extern",
            StorageClass::Register => "register",
            StorageClass::Typedef => "typedef",
        };
        write!(f, "{}", class_str)
    }
}

/// Function definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub node_id: NodeId,
    pub name: String,
    pub return_type: Type,
    pub parameters: Vec<Parameter>,
    pub body: Statement,
    pub storage_class: StorageClass,
    pub span: SourceSpan,
    pub symbol_id: Option<SymbolId>, // Filled during semantic analysis
}

/// Function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub node_id: NodeId,
    pub name: Option<String>, // Can be unnamed in function prototypes
    pub param_type: Type,
    pub span: SourceSpan,
    pub symbol_id: Option<SymbolId>, // Filled during semantic analysis
}

/// Top-level compilation unit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslationUnit {
    pub node_id: NodeId,
    pub items: Vec<TopLevelItem>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TopLevelItem {
    /// Function definition
    Function(FunctionDefinition),
    
    /// Global variable declaration
    Declaration(Declaration),
    
    /// Struct/union/enum definition
    TypeDefinition {
        name: String,
        type_def: Type,
        span: SourceSpan,
    },
}

/// Node ID generator for AST nodes
#[derive(Debug, Clone, Default)]
pub struct NodeIdGenerator {
    next_id: NodeId,
}

impl NodeIdGenerator {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }
    
    pub fn next(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcc_common::SourceLocation;

    #[test]
    fn test_type_sizes() {
        assert_eq!(Type::Char.size_in_bytes(), Some(1));
        assert_eq!(Type::Int.size_in_bytes(), Some(2)); // 16-bit int
        assert_eq!(Type::Long.size_in_bytes(), Some(4)); // 32-bit long
        assert_eq!(Type::Pointer(Box::new(Type::Int)).size_in_bytes(), Some(2)); // 16-bit pointer
        
        let array_type = Type::Array { 
            element_type: Box::new(Type::Int), 
            size: Some(10) 
        };
        assert_eq!(array_type.size_in_bytes(), Some(20)); // 10 * 2 bytes
    }

    #[test]
    fn test_type_properties() {
        assert!(Type::Int.is_integer());
        assert!(Type::Int.is_signed_integer());
        assert!(!Type::UnsignedInt.is_signed_integer());
        assert!(Type::Pointer(Box::new(Type::Int)).is_pointer());
        assert!(!Type::Int.is_pointer());
    }

    #[test]
    fn test_type_compatibility() {
        let int_type = Type::Int;
        let uint_type = Type::UnsignedInt;
        let int_ptr = Type::Pointer(Box::new(Type::Int));
        let void_ptr = Type::Pointer(Box::new(Type::Void));
        
        // Integer types are compatible
        assert!(int_type.is_assignable_from(&uint_type));
        
        // void* is compatible with any pointer
        assert!(void_ptr.is_assignable_from(&int_ptr));
        assert!(int_ptr.is_assignable_from(&void_ptr));
        
        // Different types are not compatible
        assert!(!int_type.is_assignable_from(&int_ptr));
    }

    #[test]
    fn test_node_id_generator() {
        let mut gen = NodeIdGenerator::new();
        assert_eq!(gen.next(), 0);
        assert_eq!(gen.next(), 1);
        assert_eq!(gen.next(), 2);
    }

    #[test]
    fn test_ast_node_creation() {
        let mut gen = NodeIdGenerator::new();
        let loc = SourceLocation::new_simple(1, 1);
        let span = SourceSpan::new(loc.clone(), loc);
        
        let expr = Expression {
            node_id: gen.next(),
            kind: ExpressionKind::IntLiteral(42),
            span: span.clone(),
            expr_type: Some(Type::Int),
        };
        
        assert_eq!(expr.node_id, 0);
        match expr.kind {
            ExpressionKind::IntLiteral(value) => assert_eq!(value, 42),
            _ => panic!("Expected IntLiteral"),
        }
        assert_eq!(expr.expr_type, Some(Type::Int));
    }

    #[test]
    fn test_binary_op_display() {
        assert_eq!(format!("{}", BinaryOp::Add), "+");
        assert_eq!(format!("{}", BinaryOp::Equal), "==");
        assert_eq!(format!("{}", BinaryOp::LogicalAnd), "&&");
    }

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", Type::Int), "int");
        assert_eq!(format!("{}", Type::Pointer(Box::new(Type::Char))), "char*");
        assert_eq!(format!("{}", Type::Array { 
            element_type: Box::new(Type::Int), 
            size: Some(10) 
        }), "int[10]");
    }
}