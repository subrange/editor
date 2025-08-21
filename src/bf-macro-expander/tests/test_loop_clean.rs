#[cfg(test)]
mod tests {
    use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

    #[test]
    fn test_loop_macro_clean() {
        let mut expander = MacroExpander::new();
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
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        
        println!("Expanded:\n{}", result.expanded);
        println!("\nErrors: {:?}", result.errors);
        
        // Check for no errors
        assert_eq!(result.errors.len(), 0, "Should have no errors");
        
        // Check that we have the loop label
        assert!(result.expanded.contains("loop_1:"), "Should have loop_1 label");
        
        // Check that the body was included
        assert!(result.expanded.contains("ADD R4, R4, R3"), "Should contain the ADD instruction");
    }
    
    #[test] 
    fn test_preserve_with_commas() {
        let mut expander = MacroExpander::new();
        let input = r#"
#define test(arg) {
    before arg after
}

@test({:ADD R1, R2, R3})
        "#;
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        
        println!("Expanded:\n{}", result.expanded);
        println!("\nErrors: {:?}", result.errors);
        
        // Check for no errors
        assert_eq!(result.errors.len(), 0, "Should have no errors");
        
        // Check the output
        assert!(result.expanded.contains("before ADD R1, R2, R3 after"), "Should preserve the content with commas");
    }
}