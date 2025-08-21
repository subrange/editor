#[cfg(test)]
mod tests {
    use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

    #[test]
    fn test_label_not_expanded_in_preserve() {
        let mut expander = MacroExpander::new();
        // Test that labels are NOT expanded inside preserve blocks
        let input = r#"
#define test_macro(name) {
    {label(name)}:
    {:
        {label(name)}_preserved
    }
}

@test_macro(loop)
        "#;
        
        let result = expander.expand(input, MacroExpanderOptions::default());
        
        println!("Expanded: '{}'", result.expanded);
        println!("Errors: {:?}", result.errors);
        
        // Check for errors first
        assert_eq!(result.errors.len(), 0, "Should have no errors");
        
        // Outside preserve: should just output as-is for now
        assert!(result.expanded.contains("{label(loop)}"), "Should contain {{label(loop)}}");
        // Inside preserve: should also output as-is
        assert!(result.expanded.contains("{label(loop)}_preserved"), "Should contain {{label(loop)}}_preserved");
    }
}