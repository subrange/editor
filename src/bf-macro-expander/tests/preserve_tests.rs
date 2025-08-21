use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

#[test]
fn test_preserve_with_parentheses() {
    let mut expander = MacroExpander::new();
    
    let input = r#"
#define asm_test {preserve(MOV R1, R0;)}
@asm_test
"#;
    
    let options = MacroExpanderOptions {
        generate_source_map: false,
        collapse_empty_lines: true,
        strip_comments: true,
        enable_circular_dependency_detection: false,
    };
    
    let result = expander.expand(input, options);
    assert_eq!(result.expanded.trim(), "MOV R1, R0;");
    assert!(result.errors.is_empty());
}

#[test]
fn test_preserve_shorthand() {
    let mut expander = MacroExpander::new();
    
    let input = r#"
#define asm_shorthand {: MOV R3, R4; }
@asm_shorthand
"#;
    
    let options = MacroExpanderOptions {
        generate_source_map: false,
        collapse_empty_lines: true,
        strip_comments: true,
        enable_circular_dependency_detection: false,
    };
    
    let result = expander.expand(input, options);
    assert_eq!(result.expanded.trim(), "MOV R3, R4;");
    assert!(result.errors.is_empty());
}

#[test]
fn test_preserve_with_parameters() {
    let mut expander = MacroExpander::new();
    
    let input = r#"
#define asm_2_test(reg1, reg2) {
  {: MOV reg1, reg2 }
}
@asm_2_test(R1, R2);
"#;
    
    let options = MacroExpanderOptions {
        generate_source_map: false,
        collapse_empty_lines: true,
        strip_comments: true,
        enable_circular_dependency_detection: false,
    };
    
    let result = expander.expand(input, options);
    // The macro expands to "  MOV R1, R2 " (with spaces), then the ; follows on the same line
    assert_eq!(result.expanded.trim(), "MOV R1, R2;");
    assert!(result.errors.is_empty());
}

#[test]
fn test_preserve_nested_braces() {
    let mut expander = MacroExpander::new();
    
    let input = r#"
#define complex_asm {: if (x > 0) { MOV R1, R2; } else { MOV R3, R4; } }
@complex_asm
"#;
    
    let options = MacroExpanderOptions {
        generate_source_map: false,
        collapse_empty_lines: true,
        strip_comments: true,
        enable_circular_dependency_detection: false,
    };
    
    let result = expander.expand(input, options);
    assert_eq!(result.expanded.trim(), "if (x > 0) { MOV R1, R2; } else { MOV R3, R4; }");
    assert!(result.errors.is_empty());
}

#[test]
fn test_preserve_direct_invocation() {
    let mut expander = MacroExpander::new();
    
    let input = "{preserve(ADD R5, R6;)}";
    
    let options = MacroExpanderOptions {
        generate_source_map: false,
        collapse_empty_lines: true,
        strip_comments: true,
        enable_circular_dependency_detection: false,
    };
    
    let result = expander.expand(input, options);
    assert_eq!(result.expanded.trim(), "ADD R5, R6;");
    assert!(result.errors.is_empty());
}

#[test]
fn test_shorthand_direct_invocation() {
    let mut expander = MacroExpander::new();
    
    let input = "{: SUB R7, R8; }";
    
    let options = MacroExpanderOptions {
        generate_source_map: false,
        collapse_empty_lines: true,
        strip_comments: true,
        enable_circular_dependency_detection: false,
    };
    
    let result = expander.expand(input, options);
    assert_eq!(result.expanded.trim(), "SUB R7, R8;");
    assert!(result.errors.is_empty());
}