use bf_macro_expander::{MacroExpander, MacroExpanderOptions};

fn create_macro_expander() -> MacroExpander {
    MacroExpander::new()
}

#[test]
fn test_parameter_substitution_in_expression_list() {
    // This tests the fix for parameters not being substituted in expression lists
    let input = r#"#define times(code) TIMES[code]
#define mark_register(offs) @times(offs >)
@mark_register(VALUE)"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "TIMES[VALUE>]");
}

#[test]
fn test_parameter_substitution_multiline_argument() {
    // This tests that parameters are substituted in multiline arguments
    let input = r#"#define times(code) TIMES[code]
#define mark_register(offs) {
  @times(
    offs
    >
  )
}
@mark_register(VALUE)"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "TIMES[VALUE>]");
}

#[test]
fn test_nested_macro_parameter_passing() {
    // This tests that parameters are correctly passed through nested macro invocations
    let input = r#"#define inner(offs) Inner:offs:End
#define outer(offs_from_ip) @inner(offs_from_ip)
@outer(VALUE)"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "Inner:VALUE:End");
}

#[test]
fn test_hash_as_macro_invocation() {
    // This tests that # works the same as @ for macro invocation
    let input = r#"#define test(x) Result:x
#test(VALUE)"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "Result:VALUE");
}

#[test]
fn test_brace_delimited_single_line_macro() {
    // This tests that braces can be used as delimiters for single-line macros
    let input = r#"#define powers { 1, 2, 4, 8 }
@powers"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "1,2,4,8");
}

#[test]
fn test_multiline_macro_with_braces() {
    // This tests multiline macros that start with { on a new line
    let input = r#"#define test
{
  line1
  line2
}
@test"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert!(result.expanded.contains("line1"));
    assert!(result.expanded.contains("line2"));
}

#[test]
fn test_builtin_functions_as_single_tokens() {
    // This tests that {repeat, {if, etc. are recognized as single tokens
    let input = "{repeat(3, +)}";
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "+++");
}

#[test]
fn test_reverse_with_macro_returning_comma_separated() {
    // This tests that reverse works with macros that return comma-separated values
    let input = r#"#define nums 1, 2, 3
{reverse(@nums)}"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "{3, 2, 1}");
}

#[test]
fn test_multiline_argument_with_comments() {
    // This tests that comments and newlines are handled correctly in arguments
    let input = r#"{repeat(
  3, // repeat count
  + // the command
)}"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "+++");
}

#[test]
fn test_complex_macro_like_cpu_bfm() {
    // This is a simplified version of the cpu.bfm pattern that was failing
    let input = r#"#define times(code) [-code]
#define lane(l, code) l:code
#define mark_register(offs) {
  @lane(SA,
    @times(
      offs
      >
    )
  )
}
#define fetch_from_register(offs_from_ip) {
    @mark_register(offs_from_ip)
}
@fetch_from_register(VALUE)"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert!(result.expanded.contains("VALUE"));
    assert!(!result.expanded.contains("offs"));
    assert!(!result.expanded.contains("offs_from_ip"));
}