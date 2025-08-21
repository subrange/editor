#[cfg(test)]
mod tests {
    use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

    #[test]
    fn test_parse_loop_macro() {
        let mut expander = MacroExpander::new();
        // Just test that parsing completes without hanging
        let input = r#"
#define loop(counter, start, end, body) {
    {:
    LI counter, start
    {label(loop)}:
    body
    ADDI counter, counter, 1;
    BNE counter, end, {label(loop)};}
}
  
@loop(R3, 0, 10, {:ADD R4, R4, R3})
        "#;
        
        // Set normal expansion depth
        expander.max_expansion_depth = 100;
        
        println!("Starting expansion...");
        let result = expander.expand(input, MacroExpanderOptions::default());
        
        println!("Expansion completed!");
        println!("Expanded: '{}'", result.expanded);
        println!("Errors: {:?}", result.errors);
        
        // Just check it didn't error with parameter mismatch
        for error in &result.errors {
            assert!(!error.message.contains("expects 4 parameter(s), got 6"), 
                    "Should not have parameter count error");
        }
    }
}