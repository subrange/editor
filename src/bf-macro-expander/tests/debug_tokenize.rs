use bf_macro_expander::lexer::Lexer;

#[test]
fn debug_tokenize_builtin() {
    let input = "#define inc(n) {repeat(n, +)}";
    
    let mut lexer = Lexer::new(input, false);
    let tokens = lexer.tokenize();
    
    println!("Input: {}", input);
    println!("Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?} = '{}'", i, token.token_type, token.value);
    }
}