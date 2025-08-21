#[cfg(test)]
mod tests {
    use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

    #[test]
    fn test_comma_in_preserve_block() {
        let mut expander = MacroExpander::new();
        // Test the exact case that user needs
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
{:;}
@loop(R5, 0, 5, {: SUB R6, R6, R5 })
        "#;
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        
        println!("Expanded:\n'{}'", result.expanded);
        println!("Errors: {:?}", result.errors);
        
        // Should have no parameter mismatch errors (the main issue was getting 6 params instead of 4)
        assert_eq!(result.errors.len(), 0, "Should not have parameter mismatch errors");
        
        // Check that both loops expanded correctly with unique labels
        assert!(result.expanded.contains("loop_1:"), "First loop should have loop_1 label");
        assert!(result.expanded.contains("loop_2:"), "Second loop should have loop_2 label");  
        assert!(result.expanded.contains("ADD R4, R4, R3"));
        assert!(result.expanded.contains("SUB R6, R6, R5"));
    }
}