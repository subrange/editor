#[cfg(test)]
mod tests {
    use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

    #[test]
    fn test_label_unique_per_invocation() {
        let mut expander = MacroExpander::new();
        // Test that labels are unique per macro invocation
        let input = r#"
#define test_loop(name) {
    {label(loop)}:
    name
    BNE {label(loop)}
}

@test_loop(first)
@test_loop(second)
        "#;
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        
        println!("Expanded:\n{}", result.expanded);
        println!("Errors: {:?}", result.errors);
        
        // Check for no errors
        assert_eq!(result.errors.len(), 0, "Should have no errors");
        
        // First invocation should have loop_1
        assert!(result.expanded.contains("loop_1:"), "First invocation should have loop_1:");
        assert!(result.expanded.contains("BNEloop_1") || result.expanded.contains("BNE loop_1"), "First invocation should have BNE with loop_1");
        
        // Second invocation should have loop_2  
        assert!(result.expanded.contains("loop_2:"), "Second invocation should have loop_2:");
        assert!(result.expanded.contains("BNEloop_2") || result.expanded.contains("BNE loop_2"), "Second invocation should have BNE with loop_2");
    }
    
    #[test]
    fn test_label_same_within_invocation() {
        let mut expander = MacroExpander::new();
        // Test that the same label name produces the same output within one invocation
        let input = r#"
#define test_loop() {
    {label(start)}:
    ADD
    JMP {label(start)}
    {label(end)}:
}

@test_loop()
        "#;
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        
        println!("Expanded:\n{}", result.expanded);
        
        // Should have start_1 appearing twice (same label)
        let start_count = result.expanded.matches("start_1").count();
        assert_eq!(start_count, 2, "start_1 should appear exactly twice");
        
        // Should have end_2 appearing once (different label gets different number)
        assert!(result.expanded.contains("end_2:"), "Should have end_2:");
    }
}