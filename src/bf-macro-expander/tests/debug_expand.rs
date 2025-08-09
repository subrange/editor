use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

#[test]
fn debug_multiline_macro_expansion() {
    let input = r#"#define inc(n) {repeat(n, +)}
#define dec(n) {repeat(n, -)}
#define test {
  @inc(3)
  @dec(2)
}
#test"#;
    
    let mut expander = MacroExpander::new();
    let options = MacroExpanderOptions {
        generate_source_map: false,
        strip_comments: true,
        collapse_empty_lines: true,
        enable_circular_dependency_detection: false,
    };
    
    let result = expander.expand(input, options);
    
    println!("Errors: {:?}", result.errors);
    println!("Expanded: '{}'", result.expanded);
    
    println!("\nMacro definitions:");
    for m in &result.macros {
        println!("  {}: '{}'", m.name, m.body);
    }
    
    assert_eq!(result.expanded, "+++--");
}