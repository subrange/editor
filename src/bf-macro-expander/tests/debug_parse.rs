use bf_macro_expander::lexer::Lexer;
use bf_macro_expander::parser::Parser;
use bf_macro_expander::ast::{StatementNode, ContentNode};

#[test]
fn debug_multiline_macro_parsing() {
    let input = "#define test {\n  @inc(3)\n  @dec(2)\n}\n#test";
    
    let mut lexer = Lexer::new(input, false);
    let tokens = lexer.tokenize();
    
    println!("Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?} = '{}'", i, token.token_type, token.value.replace('\n', "\\n"));
    }
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    println!("\nParsed macros:");
    for stmt in &result.ast.statements {
        if let StatementNode::MacroDefinition(def) = stmt {
            println!("  Macro '{}' body has {} nodes:", def.name, def.body.len());
            for (i, node) in def.body.iter().enumerate() {
                match node {
                    ContentNode::Text(t) => {
                        println!("    {}: Text = '{}'", i, t.value.replace('\n', "\\n").replace(' ', "·"));
                    }
                    ContentNode::MacroInvocation(m) => {
                        println!("    {}: MacroInvocation = '{}'", i, m.name);
                    }
                    ContentNode::BuiltinFunction(b) => {
                        println!("    {}: BuiltinFunction = '{:?}'", i, b.name);
                    }
                    ContentNode::BrainfuckCommand(c) => {
                        println!("    {}: BrainfuckCommand = '{}'", i, c.commands);
                    }
                }
            }
        }
    }
}