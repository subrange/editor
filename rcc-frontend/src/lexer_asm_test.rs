#[cfg(test)]
mod asm_tests {
    use crate::lexer::{Lexer, TokenType};
    
    #[test]
    fn test_asm_keyword() {
        let mut lexer = Lexer::new("__asm__ asm");
        let tokens = lexer.tokenize().unwrap();
        
        println!("Tokens:");
        for token in &tokens {
            println!("  {:?}", token.token_type);
        }
        
        assert!(matches!(tokens[0].token_type, TokenType::Asm));
        assert!(matches!(tokens[1].token_type, TokenType::Asm));
    }
}