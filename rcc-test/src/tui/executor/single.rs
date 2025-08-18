use std::path::Path;
use std::time::Instant;
use anyhow::Result;
use crate::tui::app::{TuiApp, AppMode, TestResult};
use crate::compiler::compile_c_file;
use crate::config::{RunConfig, Backend, TestCase};

pub fn run_single_test(app: &mut TuiApp, test_name: &str) -> Result<()> {
    app.clear_output();
    app.append_output(&format!("Running test: {test_name}\n"));
    app.append_output(&("-".repeat(60)));
    app.append_output("\n");
    
    // Auto-switch to Output tab when running a test
    app.selected_tab = 3;
    
    app.running_test = Some(test_name.to_string());
    app.mode = AppMode::Running;

    let start = Instant::now();
    
    // Get test details
    let test = app.get_selected_test_details();
    
    // Build and run the test
    match compile_and_run_test(app, test_name, test.as_ref()) {
        Ok(output) => {
            let duration = start.elapsed().as_millis();
            
            // Check if test passed
            let passed = if let Some(ref test) = test {
                if let Some(expected) = &test.expected {
                    output.trim() == expected.trim()
                } else {
                    true // No expected output means just check it compiles/runs
                }
            } else {
                true
            };

            let result = TestResult {
                passed,
                output: output.clone(),
                expected: test.as_ref().and_then(|t| t.expected.clone()),
                duration_ms: duration,
            };

            app.append_output(&format!("\nOutput:\n{output}\n"));
            app.append_output(&format!("\n{}\n", "-".repeat(60)));
            
            if passed {
                app.append_output(&format!("✓ Test PASSED in {duration}ms\n"));
            } else {
                app.append_output(&format!("✗ Test FAILED in {duration}ms\n"));
                if let Some(test) = test {
                    if let Some(expected) = &test.expected {
                        app.append_output(&format!("\nExpected:\n{expected}\n"));
                    }
                }
            }

            app.record_test_result(test_name.to_string(), result);
        }
        Err(e) => {
            app.append_output(&format!("\n✗ Error: {e}\n"));
            
            let result = TestResult {
                passed: false,
                output: format!("Error: {e}"),
                expected: test.as_ref().and_then(|t| t.expected.clone()),
                duration_ms: start.elapsed().as_millis(),
            };
            
            app.record_test_result(test_name.to_string(), result);
        }
    }

    app.mode = AppMode::Normal;
    Ok(())
}

fn compile_and_run_test(app: &TuiApp, test_name: &str, test: Option<&TestCase>) -> Result<String> {
    // Find test file path
    let test_path = if let Some(test) = test {
        test.file.clone()
    } else {
        // Try to find in the test config
        app.test_config.tests
            .iter()
            .find(|t| t.file.file_stem().and_then(|s| s.to_str()) == Some(test_name))
            .map(|t| t.file.clone())
            .ok_or_else(|| anyhow::anyhow!("Test not found"))?
    };

    let actual_test_path = if test_path.is_relative() && !test_path.starts_with("c-test") {
        Path::new("c-test").join(&test_path)
    } else {
        test_path.clone()
    };

    // Use the same compile_c_file function that the main test runner uses
    // This properly handles preprocessing and compiler errors/panics
    let run_config = RunConfig {
        backend: Backend::Rvm,
        timeout_secs: app.timeout_secs,
        bank_size: app.bank_size,
        verbose: false,
        no_cleanup: true,
        parallel: false,
        debug_mode: false,
        frequency: None,
    };
    
    let use_runtime = test.map(|t| t.use_runtime).unwrap_or(true);
    
    // This function properly handles compiler panics/errors
    let compilation_result = match compile_c_file(
        &actual_test_path,
        &app.tools,
        &run_config,
        use_runtime,
    ) {
        Ok(result) => result,
        Err(e) => {
            // Handle errors (like file not found) without propagating
            return Err(anyhow::anyhow!("Failed to compile: {}", e));
        }
    };
    
    if !compilation_result.success {
        // Return the error message from compilation
        if let Some(error_msg) = compilation_result.error_message {
            return Err(anyhow::anyhow!("{}", error_msg));
        } else {
            return Err(anyhow::anyhow!("Compilation failed"));
        }
    }
    
    Ok(compilation_result.output)
}