#[cfg(test)]
mod tests {
    use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

    #[test]
    fn test_for_with_index() {
        let mut expander = MacroExpander::new();
        let input = r#"
#define print_indexed(arr) {for(val, idx in arr, @print(idx, val){br})}
#define print(i, v) Index: i Value: v

@print_indexed({A, B, C})
"#;
        let options = MacroExpanderOptions {
            generate_source_map: false,
            collapse_empty_lines: true,
            strip_comments: true,
            enable_circular_dependency_detection: false,
        };
        
        let result = expander.expand(input, options);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        
        let expected = "Index: 0 Value: A\nIndex: 1 Value: B\nIndex: 2 Value: C";
        assert_eq!(result.expanded.trim(), expected);
    }

    #[test]
    fn test_for_with_index_string() {
        let mut expander = MacroExpander::new();
        let input = r#"
#define print_chars(str) {for(ch, i in str, [i]:ch{br})}

@print_chars("ABC")
"#;
        let options = MacroExpanderOptions {
            generate_source_map: false,
            collapse_empty_lines: true,
            strip_comments: true,
            enable_circular_dependency_detection: false,
        };
        
        let result = expander.expand(input, options);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        
        let expected = "[0]:65\n[1]:66\n[2]:67";
        assert_eq!(result.expanded.trim(), expected);
    }

    #[test]
    fn test_for_without_index_still_works() {
        let mut expander = MacroExpander::new();
        let input = r#"
#define print_values(arr) {for(val in arr, Value: val{br})}

@print_values({X, Y, Z})
"#;
        let options = MacroExpanderOptions {
            generate_source_map: false,
            collapse_empty_lines: true,
            strip_comments: true,
            enable_circular_dependency_detection: false,
        };
        
        let result = expander.expand(input, options);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        
        let expected = "Value: X\nValue: Y\nValue: Z";
        assert_eq!(result.expanded.trim(), expected);
    }

    #[test]
    fn test_for_with_tuple_and_index() {
        let mut expander = MacroExpander::new();
        let input = r#"
#define print_pairs(arr) {for((a, b), idx in arr, [idx]: a=a b=b{br})}

@print_pairs({{1, 2}, {3, 4}, {5, 6}})
"#;
        let options = MacroExpanderOptions {
            generate_source_map: false,
            collapse_empty_lines: true,
            strip_comments: true,
            enable_circular_dependency_detection: false,
        };
        
        let result = expander.expand(input, options);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        
        let expected = "[0]: a=1 b=2\n[1]: a=3 b=4\n[2]: a=5 b=6";
        assert_eq!(result.expanded.trim(), expected);
    }
}