use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;
use anyhow::Result;
use crossterm::{cursor, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{backend::Backend, Terminal};
use crate::tui::app::TuiApp;
use crate::tui::event::EventHandler;
use crate::compiler::compile_c_file;
use crate::config::{RunConfig, Backend as CompilerBackend};

pub fn debug_test<B: Backend>(
    app: &mut TuiApp,
    terminal: &mut Terminal<B>,
    events: &EventHandler,
) -> Result<()> {
    if let Some(test_name) = app.get_selected_test_name() {
        if let Some(test_path) = app.get_selected_test_path() {
            // Temporarily exit TUI to run debugger
            app.append_output(&format!("\nLaunching debugger for: {test_name}\n"));
            app.append_output("(TUI will resume after debugger exits)\n");
            
            // We need to compile first
            let actual_test_path = if test_path.is_relative() && !test_path.starts_with("c-test") {
                Path::new("c-test").join(&test_path)
            } else {
                test_path.clone()
            };

            // Use the proper compile_c_file function to handle compilation
            let run_config = RunConfig {
                backend: CompilerBackend::Rvm,
                timeout_secs: 30,
                bank_size: app.bank_size,
                verbose: false,
                no_cleanup: true,
                parallel: false,
                debug_mode: false,
                frequency: None,
            };
            
            // Get test details to determine if runtime is needed
            let test = app.get_selected_test_details();
            let use_runtime = test.as_ref().map(|t| t.use_runtime).unwrap_or(true);
            
            // Compile the test using the same function as the test runner
            let compilation_result = compile_c_file(
                &actual_test_path,
                &app.tools,
                &run_config,
                use_runtime,
            );
            
            match compilation_result {
                Ok(result) if !result.success => {
                    app.append_output(&format!("Compilation failed: {}\n", 
                        result.error_message.unwrap_or_else(|| "Unknown error".to_string())));
                    return Ok(());
                }
                Err(e) => {
                    app.append_output(&format!("Compilation error: {e}\n"));
                    return Ok(());
                }
                _ => {} // Success, continue
            }
            
            // The binary should now exist at the expected location
            let bin_file = app.tools.build_dir.join(format!("{test_name}.bin"));

            // Now run with debugger - need to exit TUI temporarily
            // Pause the event handler FIRST
            events.pause();
            
            // Properly suspend the terminal
            terminal.show_cursor()?;
            disable_raw_mode()?;
            execute!(
                io::stdout(),
                LeaveAlternateScreen,
                cursor::Show,
            )?;
            io::stdout().flush()?;

            // Run debugger (bin_file should exist now from compilation)
            let status = std::process::Command::new(&app.tools.rvm)
                .arg(&bin_file)
                .arg("-t")
                .status()?;

            // Properly restore the terminal
            // Small delay to ensure terminal processes the mode change
            std::thread::sleep(Duration::from_millis(100));
            
            enable_raw_mode()?;
            execute!(
                io::stdout(),
                EnterAlternateScreen,
                cursor::Hide,
            )?;
            terminal.hide_cursor()?;
            
            // Clear any events that might have been queued
            while events.rx.try_recv().is_ok() {
                // Discard any queued events
            }
            
            // Resume the event handler
            events.resume();
            
            // Clear and force complete redraw
            terminal.clear()?;
            terminal.draw(|f| crate::tui::ui::draw(f, app))?;

            if !status.success() {
                app.append_output("Debugger exited with error\n");
            } else {
                app.append_output("Debugger session ended\n");
            }
        }
    }
    Ok(())
}