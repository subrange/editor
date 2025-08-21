#[cfg(test)]
mod tests {
    use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

    #[test]
    fn test_label_counter_resets_between_expansions() {
        let mut expander = MacroExpander::new();
        
        // First expansion
        let input1 = "{label test}";
        let result1 = expander.expand(input1, MacroExpanderOptions::default());
        assert_eq!(result1.expanded, "test_1", "First expansion should use test_1");
        
        // Second expansion - counter should reset
        let input2 = "{label test}";
        let result2 = expander.expand(input2, MacroExpanderOptions::default());
        assert_eq!(result2.expanded, "test_1", "Second expansion should also use test_1, not test_2");
        
        // Third expansion with multiple labels
        let input3 = "{label foo} {label bar}";
        let result3 = expander.expand(input3, MacroExpanderOptions::default());
        assert_eq!(result3.expanded, "foo_1 bar_2", "Should start from 1 again");
    }
    
    #[test]
    fn test_label_counter_within_single_expansion() {
        let mut expander = MacroExpander::new();
        
        // Multiple labels in one expansion should increment
        let input = r#"
#define test() {
    {label loop}
    {label loop}
    {label other}
}

@test()
@test()
        "#;
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        println!("Expanded: '{}'", result.expanded);
        
        // First macro invocation gets loop_1 and other_2
        assert!(result.expanded.contains("loop_1"));
        assert!(result.expanded.contains("other_2"));
        
        // Second macro invocation gets loop_3 and other_4  
        assert!(result.expanded.contains("loop_3"));
        assert!(result.expanded.contains("other_4"));
    }
}