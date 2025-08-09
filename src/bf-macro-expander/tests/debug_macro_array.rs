use bf_macro_expander::lexer::Lexer;
use bf_macro_expander::parser::Parser;
use bf_macro_expander::ast::{StatementNode, ContentNode};

#[test]
fn debug_macro_array_body() {
    let input = "#define POWERS {1, 2, 4, 8}";
    
    let mut lexer = Lexer::new(input, false);
    let tokens = lexer.tokenize();
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    for stmt in &result.ast.statements {
        if let StatementNode::MacroDefinition(def) = stmt {
            println!("Macro '{}' body has {} nodes:", def.name, def.body.len());
            for (i, node) in def.body.iter().enumerate() {
                match node {
                    ContentNode::Text(t) => {
                        println!("  {}: Text = '{}'", i, t.value);
                    }
                    ContentNode::MacroInvocation(m) => {
                        println!("  {}: MacroInvocation = '{}'", i, m.name);
                    }
                    ContentNode::BuiltinFunction(b) => {
                        println!("  {}: BuiltinFunction = '{:?}'", i, b.name);
                    }
                    ContentNode::BrainfuckCommand(c) => {
                        println!("  {}: BrainfuckCommand = '{}'", i, c.commands);
                    }
                }
            }
        }
    }
}