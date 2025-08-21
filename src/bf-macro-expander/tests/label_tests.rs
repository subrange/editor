#[cfg(test)]
mod label_tests {
    use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

    #[test]
    fn test_simple_label() {
        let mut expander = MacroExpander::new();
        let input = "{: {label(foo)}:}";
        let result = expander.expand(input, MacroExpanderOptions::default());
        
        println!("Expanded: '{}'", result.expanded);
        println!("Errors: {:?}", result.errors);
        
        assert_eq!(result.expanded.trim(), "foo_1:");
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_label_in_macro() {
        let mut expander = MacroExpander::new();
        let input = r#"
#define test_macro(x) {
    {: {label(loop)}: x }
}

@test_macro({:ADD R1, R2})
@test_macro({:SUB R3, R4})
        "#;
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        let expanded = result.expanded.trim();
        
        println!("Expanded: '{}'", expanded);
        println!("Errors: {:?}", result.errors);
        
        assert!(expanded.contains("loop_1: ADD R1, R2"));
        assert!(expanded.contains("loop_2: SUB R3, R4"));
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_loop_macro_with_labels() {
        let mut expander = MacroExpander::new();
        // Simpler test - just test that labels work in preserve blocks
        let input = r#"
#define test() {
    {: start {label(loop)}: end}
}

@test()
        "#;
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        let expanded = result.expanded.trim();
        
        println!("Expanded: '{}'", expanded);
        println!("Errors: {:?}", result.errors);
        
        assert!(expanded.contains("start loop_1: end"));
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_multiple_labels_in_macro() {
        let mut expander = MacroExpander::new();
        // Test with a real loop structure
        let input = r#"
#define loop(body) {
    {: LI R1, 0
    {label(loop_start)}:
    body
    ADDI R1, R1, 1
    BNE R1, 10, {label(loop_start)}}
}

@loop({: ADD R2, R2, R1})
        "#;
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        let expanded = result.expanded.trim();
        
        println!("Expanded: '{}'", expanded);
        println!("Errors: {:?}", result.errors);
        
        assert!(expanded.contains("loop_start_1:"));
        assert!(expanded.contains("BNE R1, 10, loop_start_1"));
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_label_with_parameter_substitution() {
        let mut expander = MacroExpander::new();
        let input = r#"
#define jump_macro(label_name, reg) {
    {: {label(label_name)}:
    ADD reg, reg
    JMP {label(label_name)}}
}

@jump_macro(my_loop, R5)
@jump_macro(other_loop, R7)
        "#;
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        let expanded = result.expanded.trim();
        
        // Labels should be based on parameter values
        assert!(expanded.contains("my_loop_1:"));
        assert!(expanded.contains("JMP my_loop_1"));
        assert!(expanded.contains("other_loop_2:"));
        assert!(expanded.contains("JMP other_loop_2"));
        
        assert_eq!(result.errors.len(), 0);
    }
}