#[cfg(test)]
mod tests {
    use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

    #[test]
    fn test_br_basic() {
        let mut expander = MacroExpander::new();
        let input = "before{br()}after";
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        
        println!("Input: '{}'", input);
        println!("Expanded: '{}'", result.expanded);
        
        assert_eq!(result.expanded, "before\nafter");
    }
    
    #[test]
    fn test_loop_with_br() {
        let mut expander = MacroExpander::new();
        let input = r#"
#define loop(counter, start, end, body) {
    {: LI counter, start }{br()}
    {label(loop)}:{br()}
    {:body}{br()}
    {:ADDI counter, counter, 1}{br()}
    {:BNE counter, end, }{label(loop)}
}

@loop(R3, 0, 10, {:ADD R4, R4, R3})
        "#;
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        
        println!("Expanded:\n{}", result.expanded);
        println!("\nErrors: {:?}", result.errors);
        
        // Check for no errors
        assert_eq!(result.errors.len(), 0, "Should have no errors");
        
        // Check that we have proper line breaks
        let lines: Vec<_> = result.expanded.lines().collect();
        assert!(lines.len() > 3, "Should have multiple lines");
        
        // Check for the loop label on its own line
        assert!(lines.iter().any(|line| line.trim() == "loop_1:"), "Should have loop_1: on its own line");
    }
}