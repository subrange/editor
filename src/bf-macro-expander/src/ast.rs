use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ASTPosition {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgramNode {
    pub statements: Vec<StatementNode>,
    pub position: ASTPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum StatementNode {
    MacroDefinition(MacroDefinitionNode),
    CodeLine(CodeLineNode),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroDefinitionNode {
    pub name: String,
    pub parameters: Option<Vec<String>>,
    pub body: Vec<BodyNode>,
    pub position: ASTPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeLineNode {
    pub content: Vec<ContentNode>,
    pub position: ASTPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ContentNode {
    BrainfuckCommand(BrainfuckCommandNode),
    MacroInvocation(MacroInvocationNode),
    BuiltinFunction(BuiltinFunctionNode),
    Text(TextNode),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrainfuckCommandNode {
    pub commands: String,
    pub position: ASTPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroInvocationNode {
    pub name: String,
    pub arguments: Option<Vec<ExpressionNode>>,
    pub position: ASTPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuiltinFunctionNode {
    pub name: BuiltinFunction,
    pub arguments: Vec<ExpressionNode>,
    pub position: ASTPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BuiltinFunction {
    Repeat,
    If,
    For,
    Reverse,
    Preserve,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArrayLiteralNode {
    pub elements: Vec<ExpressionNode>,
    pub position: ASTPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ExpressionNode {
    Number(NumberNode),
    Identifier(IdentifierNode),
    MacroInvocation(MacroInvocationNode),
    BuiltinFunction(BuiltinFunctionNode),
    ExpressionList(ExpressionListNode),
    Text(TextNode),
    BrainfuckCommand(BrainfuckCommandNode),
    ArrayLiteral(ArrayLiteralNode),
    TuplePattern(TuplePatternNode),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumberNode {
    pub value: i64,
    pub position: ASTPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdentifierNode {
    pub name: String,
    pub position: ASTPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TuplePatternNode {
    pub elements: Vec<String>,
    pub position: ASTPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpressionListNode {
    pub expressions: Vec<ContentNode>,
    pub position: ASTPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextNode {
    pub value: String,
    pub position: ASTPosition,
}

pub type BodyNode = ContentNode;