#[cfg(test)]
mod tests {
    use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

    #[test]
    fn test_label_direct() {
        let mut expander = MacroExpander::new();
        // Test label directly without macros
        let input = "{label(loop)}:";
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        
        println!("Input: '{}'", input);
        println!("Expanded: '{}'", result.expanded);
        println!("Errors: {:?}", result.errors);
        
        // Should output as-is
        assert!(result.expanded.contains("{label(loop)}"));
    }
    
    #[test]
    fn test_label_in_preserve() {
        let mut expander = MacroExpander::new();
        // Test label inside preserve block
        let input = "{:text before {label(loop)} text after}";
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        
        println!("Input: '{}'", input);
        println!("Expanded: '{}'", result.expanded);
        println!("Errors: {:?}", result.errors);
        
        // Should preserve everything including the label
        assert!(result.expanded.contains("text before"));
        assert!(result.expanded.contains("{label(loop)}"));
        assert!(result.expanded.contains("text after"));
    }
}