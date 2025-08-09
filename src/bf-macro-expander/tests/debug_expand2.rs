use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

#[test]
fn debug_simple_builtin() {
    // First test just the builtin function expansion
    let input = "{repeat(3, +)}";
    
    let mut expander = MacroExpander::new();
    let options = MacroExpanderOptions {
        generate_source_map: false,
        strip_comments: true,
        collapse_empty_lines: true,
        enable_circular_dependency_detection: false,
    };
    
    let result = expander.expand(input, options);
    
    println!("Input: {}", input);
    println!("Errors: {:?}", result.errors);
    println!("Expanded: '{}'", result.expanded);
    
    assert_eq!(result.expanded, "+++");
}

#[test]
fn debug_macro_with_builtin() {
    // Now test a macro that contains a builtin
    let input = r#"#define inc(n) {repeat(n, +)}
@inc(3)"#;
    
    let mut expander = MacroExpander::new();
    let options = MacroExpanderOptions {
        generate_source_map: false,
        strip_comments: true,
        collapse_empty_lines: true,
        enable_circular_dependency_detection: false,
    };
    
    let result = expander.expand(input, options);
    
    println!("Input: {}", input);
    println!("Errors: {:?}", result.errors);
    println!("Expanded: '{}'", result.expanded);
    
    println!("\nMacro definitions:");
    for m in &result.macros {
        println!("  {}: '{}'", m.name, m.body);
    }
    
    assert_eq!(result.expanded, "+++");
}