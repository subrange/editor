use bf_macro_expander::{create_macro_expander, MacroExpanderOptions, MacroExpansionErrorType};

#[test]
fn test_reverse_array_literal() {
    let input = "{reverse({1, 2, 3})}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded, "{3, 2, 1}");
}

#[test]
fn test_reverse_array_with_text() {
    let input = "{reverse({a, b, c, d})}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded, "{d, c, b, a}");
}

#[test]
fn test_reverse_with_for_loops() {
    let input = "{for(i in {reverse({1, 2, 3})}, i)}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded, "321");
}

#[test]
fn test_reverse_array_from_macro() {
    let input = "#define nums {1, 2, 3, 4, 5}\n{reverse(@nums)}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "{5, 4, 3, 2, 1}");
}

#[test]
fn test_reverse_empty_array() {
    let input = "{reverse({})}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded, "{}");
}

#[test]
fn test_reverse_single_element() {
    let input = "{reverse({42})}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded, "{42}");
}

#[test]
fn test_reverse_error_non_array() {
    let input = "{reverse(123)}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert!(result.errors.len() > 0);
    assert!(result.errors[0].message.contains("array literal"));
}

#[test]
fn test_reverse_error_wrong_args() {
    let input = "{reverse({1, 2}, {3, 4})}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert!(result.errors.len() > 0);
    assert!(result.errors[0].message.contains("expects exactly 1 argument"));
}

#[test]
fn test_for_loop_array_literal() {
    let input = "{for(i in {1, 2, 3}, +)}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded, "+++");
}

#[test]
fn test_for_loop_with_macro_body() {
    let input = "#define inc(n) {repeat(n, +)}\n{for(v in {1, 2, 3}, @inc(v))}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "++++++"); // 1 + 2 + 3 = 6 pluses
}

#[test]
fn test_for_loop_complex_body() {
    let input = "#define set(n) [-]{repeat(n, +)}\n{for(v in {3, 5}, @set(v) >)}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "[-]+++>[-]+++++>");
}

#[test]
fn test_for_loop_macro_returns_array() {
    let input = "#define nums {1, 2, 3, 4, 5}\n{for(x in @nums, <)}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "<<<<<");
}

#[test]
fn test_nested_for_loops() {
    let input = "{for(i in {1, 2}, {for(j in {3, 4}, i j)})}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded, "13142324");
}

#[test]
fn test_for_loop_empty_array() {
    let input = "{for(i in {}, +)}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded, "");
}

#[test]
fn test_macro_with_leading_whitespace() {
    let input = "  #define test +\n@test";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "+");
}

#[test]
fn test_macro_with_tabs() {
    let input = "\t#define test -\n@test";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "-");
}

#[test]
fn test_undefined_macro_in_definition() {
    let input = "#define a @unknown";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].error_type, MacroExpansionErrorType::Undefined);
    assert_eq!(result.errors[0].message, "Macro 'unknown' is not defined");
}

#[test]
fn test_forward_references() {
    let input = "#define a @b\n#define b +";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.macros.len(), 2);
}

#[test]
fn test_parameter_validation() {
    let input = "#define inc(n) {repeat(n, +)}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
}

#[test]
fn test_nested_macro_invocations() {
    let input = "#define outer @inner(5)\n#define inner(x) {repeat(x, +)}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
}

#[test]
fn test_multiple_undefined_macros() {
    let input = "#define test @foo @bar @baz";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 3);
    assert!(result.errors.iter().all(|e| e.error_type == MacroExpansionErrorType::Undefined));
}

#[test]
fn test_parameter_substitution_in_nested_calls() {
    let input = r#"#define next(n) {repeat(n, >)}
#define L_SCRATCH_A 1
#define lane(n) @next(n)
#define lane_sA @lane(@L_SCRATCH_A)
@lane_sA"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), ">");
}

#[test]
fn test_parameter_substitution_in_builtin() {
    let input = "#define move(dir, count) {repeat(count, dir)}\n@move(>, 3)";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), ">>>");
}

#[test]
fn test_complex_parameter_substitution() {
    let input = r#"#define A 2
#define B @A
#define fn(x) {repeat(x, +)}
#define indirect(y) @fn(y)
@indirect(@B)"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "++");
}

#[test]
fn test_if_builtin_with_parameters() {
    let input = "#define cond(x) {if(x, >, <)}\n@cond(1)";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), ">");
}

#[test]
fn test_if_builtin_zero_condition() {
    let input = "#define cond(x) {if(x, >, <)}\n@cond(0)";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "<");
}

#[test]
fn test_nested_if_conditions() {
    let input = r#"#define A 1
#define B 0
#define test {if(@A, {if(@B, +, -)}, *)}
@test"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "-");
}

#[test]
fn test_circular_dependency_detection() {
    let input = r#"#define a @b
#define b @c
#define c @a
@a"#;
    
    let mut expander = create_macro_expander();
    let options = MacroExpanderOptions {
        enable_circular_dependency_detection: true,
        ..Default::default()
    };
    let result = expander.expand(input, options);
    
    assert!(result.errors.len() > 0);
    assert!(result.errors.iter().any(|e| e.error_type == MacroExpansionErrorType::CircularDependency));
}

#[test]
fn test_empty_macro_bodies() {
    let input = "#define empty\n#define uses_empty @empty+@empty\n@uses_empty";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "+");
}

#[test]
fn test_parameter_mismatch_no_params_with_args() {
    let input = "#define goword >\n@goword(5)";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].error_type, MacroExpansionErrorType::ParameterMismatch);
    assert_eq!(result.errors[0].message, "Macro 'goword' expects 0 parameter(s), got 1");
}

#[test]
fn test_parameter_mismatch_with_params_no_args() {
    let input = "#define move(n) {repeat(n, >)}\n@move";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].error_type, MacroExpansionErrorType::ParameterMismatch);
    assert_eq!(result.errors[0].message, "Macro 'move' expects 1 parameter(s), got 0");
}

#[test]
fn test_correct_macro_invocation() {
    let input = "#define right >\n@right";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), ">");
}

#[test]
fn test_correct_macro_with_params() {
    let input = "#define move(n) {repeat(n, >)}\n@move(5)";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), ">>>>>");
}

#[test]
fn test_wrong_number_of_arguments() {
    let input = "#define add(a, b) a+b\n@add(1)";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].error_type, MacroExpansionErrorType::ParameterMismatch);
    assert_eq!(result.errors[0].message, "Macro 'add' expects 2 parameter(s), got 1");
}

#[test]
fn test_source_map_generation() {
    let input = "#define inc(n) {repeat(n, +)}\n@inc(3)";
    let mut expander = create_macro_expander();
    let options = MacroExpanderOptions {
        generate_source_map: true,
        ..Default::default()
    };
    let result = expander.expand(input, options);
    
    assert_eq!(result.errors.len(), 0);
    assert!(result.source_map.is_some());
    assert!(result.source_map.unwrap().entries.len() > 0);
}

#[test]
fn test_no_source_map_by_default() {
    let input = "#define test +\n@test";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert!(result.source_map.is_none());
}

#[test]
fn test_tuple_destructuring_two_vars() {
    let input = r#"#define PROGRAM {1, 2}
#define set(a) +
#define next(b) >
{for((a, b) in {@PROGRAM}, @set(a) @next(b))}"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "+>");
}

#[test]
fn test_tuple_destructuring_array_of_tuples() {
    let input = r#"#define PAIRS {{1, 2}, {3, 4}, {5, 6}}
#define process(x, y) {repeat(x, +)}{repeat(y, -)}
{for((a, b) in @PAIRS, @process(a, b))}"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "+--+++----+++++------");
}

#[test]
fn test_tuple_destructuring_three_vars() {
    let input = r#"#define TRIPLES {{1, 2, 3}, {4, 5, 6}}
{for((x, y, z) in @TRIPLES, xyz)}"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "123456");
}

#[test]
fn test_tuple_destructuring_direct_literals() {
    let input = "{for((a, b) in {{10, 20}, {30, 40}}, a-b)}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded, "10-2030-40");
}

#[test]
fn test_nested_arrays_in_for_loops() {
    let input = r#"#define PROGRAM {{1}, {2}}
#define set(v) +
{for(a in @PROGRAM, {for(v in a, @set(v))})}"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "++");
}

#[test]
fn test_nested_arrays_with_multiple_elements() {
    let input = r#"#define ARRAYS {{1, 2}, {3, 4, 5}}
#define process(x) {repeat(x, -)}
{for(arr in @ARRAYS, {for(val in arr, @process(val))})}"#;
    
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded.trim(), "---------------"); // 1+2+3+4+5 = 15 dashes
}

#[test]
fn test_direct_nested_array_literals() {
    let input = "{for(a in {{1, 2}, {3}}, {for(v in a, v)})}";
    let mut expander = create_macro_expander();
    let result = expander.expand(input, MacroExpanderOptions::default());
    
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.expanded, "123");
}

#[test]
fn test_comment_stripping() {
    let input = r#"// Comment before
#define test + // inline comment
/* multi
   line
   comment */
@test"#;
    
    let mut expander = create_macro_expander();
    let options = MacroExpanderOptions {
        strip_comments: true,
        ..Default::default()
    };
    let result = expander.expand(input, options);
    
    assert!(!result.expanded.contains("//"));
    assert!(!result.expanded.contains("/*"));
    assert_eq!(result.expanded.trim(), "+");
}

#[test]
fn test_comment_preservation() {
    let input = "// Comment\n#define test +\n@test // usage";
    let mut expander = create_macro_expander();
    let options = MacroExpanderOptions {
        strip_comments: false,
        collapse_empty_lines: false,
        ..Default::default()
    };
    let result = expander.expand(input, options);
    
    assert!(result.expanded.contains("// Comment"));
    assert!(result.expanded.contains("// usage"));
}

#[test]
fn test_collapse_empty_lines() {
    let input = "#define a +\n#define b -\n\n@a\n\n@b";
    let mut expander = create_macro_expander();
    let options = MacroExpanderOptions {
        collapse_empty_lines: true,
        ..Default::default()
    };
    let result = expander.expand(input, options);
    
    let lines: Vec<&str> = result.expanded.split('\n').filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines, vec!["+", "-"]);
}