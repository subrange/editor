use rcc_frontend::lexer::Lexer;

fn main() {
    let input = "__asm__(\"test\");";
    let mut lexer = Lexer::new(input);
    
    let tokens = lexer.tokenize().unwrap();
    for token in &tokens {
        println!("{:?}", token);
    }
}