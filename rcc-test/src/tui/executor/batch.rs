use std::sync::mpsc;
use std::thread;
use anyhow::Result;
use crate::tui::app::{TuiApp, AppMode, TestResult, TestMessage};
use crate::runner::TestRunner;
use crate::config::{RunConfig, Backend, TestCase};

pub fn run_batch_tests(app: &mut TuiApp, tests_to_run: Vec<TestCase>) -> Result<()> {
    app.clear_output();
    
    if tests_to_run.is_empty() {
        app.append_output("No tests to run.\n");
        return Ok(());
    }
    
    let test_count = tests_to_run.len();
    app.append_output(&format!("🚀 Running {} test{}...\n", 
        test_count, 
        if test_count == 1 { "" } else { "s" }
    ));
    app.append_output(&("=".repeat(60)));
    app.append_output("\n\n");
    
    // Auto-switch to Output tab when running tests
    app.selected_tab = 3;
    // Reset scroll to top to see the start of the test run
    app.output_scroll = 0;

    run_test_batch_internal(app, tests_to_run)
}

pub fn run_category_tests(app: &mut TuiApp, category_name: &str) -> Result<()> {
    // Get tests in this category
    let tests_to_run = app.get_category_tests(category_name);
    
    if tests_to_run.is_empty() {
        app.append_output(&format!("No tests in category '{category_name}'.\n"));
        return Ok(());
    }
    
    app.clear_output();
    let test_count = tests_to_run.len();
    app.append_output(&format!("📁 Running {} test{} in category '{}'...\n", 
        test_count,
        if test_count == 1 { "" } else { "s" },
        category_name
    ));
    app.append_output(&("=".repeat(60)));
    app.append_output("\n\n");
    
    // Auto-switch to Output tab when running tests
    app.selected_tab = 3;
    // Reset scroll to top to see the start of the test run
    app.output_scroll = 0;
    
    run_test_batch_internal(app, tests_to_run)
}

fn run_test_batch_internal(app: &mut TuiApp, tests_to_run: Vec<TestCase>) -> Result<()> {
    // Create channel for communication
    let (tx, rx) = mpsc::channel();
    app.test_receiver = Some(rx);
    
    // Clone necessary data for the thread
    let tools = app.tools.clone();
    let bank_size = app.bank_size;
    let timeout_secs = app.timeout_secs;
    
    // Spawn thread to run tests with panic handling
    thread::spawn(move || {
        // Wrap the test execution in a catch_unwind to handle panics gracefully
        let tx_panic = tx.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Create run config - EXACT SAME as CLI
            let run_config = RunConfig {
                backend: Backend::Rvm,
                timeout_secs,
                bank_size,
                verbose: false,
                no_cleanup: true,
                parallel: true,  // Use parallel execution like CLI
                debug_mode: false,
                frequency: None,
            };
            
            // Create the same TestRunner that CLI uses
            let runner = TestRunner::new(run_config, tools);
            
            // Prepare test references for batch execution
            let test_refs: Vec<&TestCase> = tests_to_run.iter().collect();
            
            // Send a single progress message at the start
            let _ = tx.send(TestMessage::Progress(
                format!("Running {} tests in parallel...", test_refs.len())
            ));
            
            // Run ALL tests in parallel with hidden progress bar (doesn't corrupt TUI)
            let results = runner.run_test_batch_with_tui(&test_refs, true);
            
            // Convert all results at once
            let mut all_results = Vec::new();
            for (test, result) in tests_to_run.iter().zip(results.iter()) {
                let test_name = test.file.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // Convert TestRunner's TestResult to our TestResult
                let passed = result.status.is_success();
                let output = result.actual_output.clone().unwrap_or_default();
                
                let ui_result = TestResult {
                    passed,
                    output,
                    expected: test.expected.clone(),
                    duration_ms: result.duration_ms as u128,
                };
                
                all_results.push((test_name, ui_result));
            }
            
            // Send ALL results as a batch
            let _ = tx.send(TestMessage::BatchCompleted(all_results));
            
            // Send finished message
            let _ = tx.send(TestMessage::Finished);
        }));
        
        // If the test runner panicked, send an error message
        if result.is_err() {
            let _ = tx_panic.send(TestMessage::Progress(
                "ERROR: Test runner crashed unexpectedly".to_string()
            ));
            let _ = tx_panic.send(TestMessage::Finished);
        }
    });
    
    app.mode = AppMode::Running;
    Ok(())
}