use std::io::{self, Write};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use anyhow::Result;
use crossterm::{
    cursor,
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::compiler::{build_runtime, ToolPaths};
use crate::command::run_command_sync;
use crate::config::TestConfig;
use crate::tui::{
    app::{TuiApp, AppMode, FocusedPane, TestResult, MetadataField},
    event::{Event, EventHandler, KeyEvent},
    ui,
};

pub struct TuiRunner {
    app: TuiApp,
    events: EventHandler,
}

impl TuiRunner {
    pub fn new(test_config: TestConfig, tools: ToolPaths, bank_size: usize, timeout_secs: u64) -> Self {
        let app = TuiApp::new(test_config, tools, bank_size, timeout_secs);
        let events = EventHandler::new(Duration::from_millis(250));
        
        Self { app, events }
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        // Check if runtime library exists before building
        let crt0_path = self.app.tools.crt0();
        let libruntime_path = self.app.tools.libruntime();
        
        if !crt0_path.exists() || !libruntime_path.exists() {
            self.app.append_output("Building runtime library...\n");
            if let Err(e) = build_runtime(&self.app.tools, self.app.bank_size) {
                self.app.append_output(&format!("Failed to build runtime: {}\n", e));
            } else {
                self.app.append_output("Runtime library built successfully.\n\n");
            }
        } else {
            self.app.append_output("Runtime library already built.\n\n");
        }

        let res = self.run_app(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err);
        }

        Ok(())
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| ui::draw(f, &mut self.app))?;
            
            // Check for test messages if tests are running
            if let Some(receiver) = &self.app.test_receiver {
                // Use try_recv to avoid blocking
                if let Ok(msg) = receiver.try_recv() {
                    self.handle_test_message(msg);
                }
            }

            match self.events.next()? {
                Event::Input(key) => {
                    if !self.handle_input(key, terminal)? {
                        return Ok(());
                    }
                }
                Event::Tick => {
                    // Handle any background updates
                }
                Event::Mouse(_) => {
                    // Ignore mouse events for now
                }
            }
        }
    }
    
    fn handle_test_message(&mut self, msg: crate::tui::app::TestMessage) {
        use crate::tui::app::TestMessage;
        
        match msg {
            TestMessage::Started(test_name) => {
                self.app.append_output(&format!("Running: {}\n", test_name));
            }
            TestMessage::Completed(test_name, result) => {
                if result.passed {
                    self.app.append_output(&format!("  ✓ {} PASSED ({}ms)\n", test_name, result.duration_ms));
                } else {
                    self.app.append_output(&format!("  ✗ {} FAILED ({}ms)\n", test_name, result.duration_ms));
                }
                self.app.record_test_result(test_name, result);
            }
            TestMessage::BatchCompleted(results) => {
                // Process all results at once
                let mut passed = 0;
                let mut failed = 0;
                
                for (test_name, result) in results {
                    if result.passed {
                        passed += 1;
                        self.app.append_output(&format!("  ✓ {} PASSED ({}ms)\n", test_name, result.duration_ms));
                    } else {
                        failed += 1;
                        self.app.append_output(&format!("  ✗ {} FAILED ({}ms)\n", test_name, result.duration_ms));
                    }
                    self.app.record_test_result(test_name, result);
                }
                
                // Show summary
                self.app.append_output(&format!("\n{}\n", "=".repeat(60)));
                self.app.append_output(&format!("Results: {} passed, {} failed, {} total\n", 
                    passed, failed, passed + failed));
            }
            TestMessage::Progress(msg) => {
                self.app.append_output(&format!("{}\n", msg));
            }
            TestMessage::Finished => {
                self.app.test_receiver = None;
                self.app.mode = AppMode::Normal;
            }
        }
    }

    fn handle_input<B: Backend>(&mut self, key: KeyEvent, terminal: &mut Terminal<B>) -> Result<bool> {
        // Handle help scrolling first if help is open
        if self.app.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => {
                    self.app.show_help = false;
                    self.app.help_scroll = 0;
                    return Ok(true);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.app.help_scroll = self.app.help_scroll.saturating_add(1);
                    return Ok(true);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.app.help_scroll = self.app.help_scroll.saturating_sub(1);
                    return Ok(true);
                }
                KeyCode::PageDown => {
                    self.app.help_scroll = self.app.help_scroll.saturating_add(10);
                    return Ok(true);
                }
                KeyCode::PageUp => {
                    self.app.help_scroll = self.app.help_scroll.saturating_sub(10);
                    return Ok(true);
                }
                _ => return Ok(true), // Ignore other keys when help is open
            }
        }

        match self.app.mode {
            AppMode::Normal => self.handle_normal_input(key, terminal),
            AppMode::FindTest => self.handle_find_test_input(key),
            AppMode::Running => self.handle_running_input(key),
            AppMode::SelectCategory => self.handle_category_input(key),
            AppMode::AddingMetadata => self.handle_metadata_input(key),
            AppMode::ConfirmDelete => self.handle_delete_confirmation(key),
        }
    }

    fn handle_normal_input<B: Backend>(&mut self, key: KeyEvent, terminal: &mut Terminal<B>) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => return Ok(false),
            KeyCode::Char('?') => {
                self.app.show_help = true;
                self.app.help_scroll = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        // Scroll the currently selected tab
                        match self.app.selected_tab {
                            0 => self.app.source_scroll = self.app.source_scroll.saturating_add(1),
                            1 => self.app.asm_scroll = self.app.asm_scroll.saturating_add(1),
                            2 => self.app.ir_scroll = self.app.ir_scroll.saturating_add(1),
                            3 => self.app.output_scroll = self.app.output_scroll.saturating_add(1),
                            4 => self.app.details_scroll = self.app.details_scroll.saturating_add(1),
                            _ => {}
                        }
                    }
                    FocusedPane::TestList => {
                        self.app.move_selection_down();
                    }
                    _ => {}
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        // Scroll the currently selected tab
                        match self.app.selected_tab {
                            0 => self.app.source_scroll = self.app.source_scroll.saturating_sub(1),
                            1 => self.app.asm_scroll = self.app.asm_scroll.saturating_sub(1),
                            2 => self.app.ir_scroll = self.app.ir_scroll.saturating_sub(1),
                            3 => self.app.output_scroll = self.app.output_scroll.saturating_sub(1),
                            4 => self.app.details_scroll = self.app.details_scroll.saturating_sub(1),
                            _ => {}
                        }
                    }
                    FocusedPane::TestList => {
                        self.app.move_selection_up();
                    }
                    _ => {}
                }
            }
            KeyCode::Enter => {
                // Check what type of item is selected
                use crate::tui::app::SelectedItemType;
                match self.app.get_selected_item_type() {
                    SelectedItemType::Category(_) => {
                        // Toggle category expansion
                        self.app.toggle_current_category();
                    }
                    SelectedItemType::Test(_) => {
                        // Run the test
                        self.run_selected_test()?;
                    }
                    SelectedItemType::None => {}
                }
            }
            KeyCode::Char(' ') => {
                // Space also toggles category expansion
                self.app.toggle_current_category();
            }
            KeyCode::Char('d') => {
                self.debug_selected_test(terminal)?;
            }
            KeyCode::Char('r') => {
                self.run_all_visible_tests()?;
            }
            KeyCode::Char('R') => {
                // Shift+R - run all tests in current category
                self.run_category_tests()?;
            }
            KeyCode::Char('c') => {
                self.app.toggle_category_selection();
            }
            KeyCode::Char('/') => {
                self.app.start_find_test();
            }
            KeyCode::Char('m') => {
                // Add metadata to orphan test
                if self.app.is_current_test_orphan() {
                    self.app.start_adding_metadata();
                }
            }
            KeyCode::Char('x') | KeyCode::Delete => {
                // Delete test (any test, not just orphans)
                self.app.start_delete_test();
            }
            KeyCode::Char('o') => {
                // Jump to first orphan test
                self.app.jump_to_first_orphan();
            }
            KeyCode::Tab => {
                self.app.focused_pane = match self.app.focused_pane {
                    FocusedPane::TestList => FocusedPane::RightPanel,
                    FocusedPane::RightPanel => FocusedPane::TestList,
                    FocusedPane::Filter => FocusedPane::TestList,
                };
            }
            KeyCode::Char('1') => {
                self.app.selected_tab = 0;  // Source
            }
            KeyCode::Char('2') => {
                self.app.selected_tab = 1;  // ASM
            }
            KeyCode::Char('3') => {
                self.app.selected_tab = 2;  // IR
            }
            KeyCode::Char('4') => {
                self.app.selected_tab = 3;  // Output
            }
            KeyCode::Char('5') => {
                self.app.selected_tab = 4;  // Details
            }
            KeyCode::PageDown => {
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        match self.app.selected_tab {
                            0 => self.app.source_scroll = self.app.source_scroll.saturating_add(20),
                            1 => self.app.asm_scroll = self.app.asm_scroll.saturating_add(20),
                            2 => self.app.ir_scroll = self.app.ir_scroll.saturating_add(20),
                            3 => self.app.output_scroll = self.app.output_scroll.saturating_add(20),
                            4 => self.app.details_scroll = self.app.details_scroll.saturating_add(20),
                            _ => {}
                        }
                    }
                    FocusedPane::TestList => {
                        for _ in 0..10 {
                            self.app.move_selection_down();
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::PageUp => {
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        match self.app.selected_tab {
                            0 => self.app.source_scroll = self.app.source_scroll.saturating_sub(20),
                            1 => self.app.asm_scroll = self.app.asm_scroll.saturating_sub(20),
                            2 => self.app.ir_scroll = self.app.ir_scroll.saturating_sub(20),
                            3 => self.app.output_scroll = self.app.output_scroll.saturating_sub(20),
                            4 => self.app.details_scroll = self.app.details_scroll.saturating_sub(20),
                            _ => {}
                        }
                    }
                    FocusedPane::TestList => {
                        for _ in 0..10 {
                            self.app.move_selection_up();
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Home => {
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        match self.app.selected_tab {
                            0 => self.app.source_scroll = 0,
                            1 => self.app.asm_scroll = 0,
                            2 => self.app.ir_scroll = 0,
                            3 => self.app.output_scroll = 0,
                            4 => self.app.details_scroll = 0,
                            _ => {}
                        }
                    }
                    FocusedPane::TestList => {
                        self.app.selected_test = 0;
                        self.app.ensure_selection_visible();
                    }
                    _ => {}
                }
            }
            KeyCode::End => {
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        // Jump to end of current tab
                        match self.app.selected_tab {
                            3 => {
                                let total_lines = self.app.output_buffer.lines().count();
                                self.app.output_scroll = total_lines.saturating_sub(10);
                            }
                            _ => {}
                        }
                    }
                    FocusedPane::TestList => {
                        let total = self.app.filtered_tests.len() + self.app.filtered_failures.len();
                        if total > 0 {
                            self.app.selected_test = total - 1;
                            self.app.ensure_selection_visible();
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(true)
    }

    fn handle_find_test_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.app.close_find_test();
            }
            KeyCode::Enter => {
                // Jump to selected test and close finder
                self.app.jump_to_selected_search_result();
                self.app.close_find_test();
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                if !self.app.search_results.is_empty() {
                    self.app.search_selected_index = 
                        (self.app.search_selected_index + 1) % self.app.search_results.len();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.app.search_results.is_empty() {
                    if self.app.search_selected_index > 0 {
                        self.app.search_selected_index -= 1;
                    } else {
                        self.app.search_selected_index = self.app.search_results.len() - 1;
                    }
                }
            }
            KeyCode::Char(c) => {
                self.app.search_query.push(c);
                self.app.update_search_results();
            }
            KeyCode::Backspace => {
                self.app.search_query.pop();
                self.app.update_search_results();
            }
            _ => {}
        }
        Ok(true)
    }

    fn handle_running_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Cancel execution
                self.app.test_receiver = None;  // Drop receiver to stop checking for messages
                self.app.append_output("\n\n[Test execution cancelled]\n");
                self.app.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(true)
    }

    fn handle_category_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.app.show_categories = false;
                self.app.mode = AppMode::Normal;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.app.move_category_selection_down();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.app.move_category_selection_up();
            }
            KeyCode::Enter => {
                self.app.select_current_category();
                self.app.show_categories = false;
                self.app.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(true)
    }

    fn handle_delete_confirmation(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Confirm deletion
                if let Err(e) = self.app.confirm_delete_test() {
                    self.app.append_output(&format!("Failed to delete test: {}\n", e));
                } else {
                    self.app.append_output("Test deleted successfully!\n");
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                // Cancel deletion
                self.app.cancel_delete();
            }
            _ => {}
        }
        Ok(true)
    }

    fn handle_metadata_input(&mut self, key: KeyEvent) -> Result<bool> {
        
        match key.code {
            KeyCode::Esc => {
                // Cancel metadata input
                self.app.metadata_input = crate::tui::app::MetadataInput::default();
                self.app.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                // Save metadata
                if let Err(e) = self.app.save_metadata() {
                    self.app.append_output(&format!("Failed to save metadata: {}\n", e));
                } else {
                    self.app.append_output("Metadata saved successfully!\n");
                }
            }
            KeyCode::Tab => {
                // Move to next field
                self.app.metadata_input.focused_field = match self.app.metadata_input.focused_field {
                    MetadataField::ExpectedOutput => MetadataField::Description,
                    MetadataField::Description => MetadataField::UseRuntime,
                    MetadataField::UseRuntime => MetadataField::IsKnownFailure,
                    MetadataField::IsKnownFailure => MetadataField::ExpectedOutput,
                };
            }
            KeyCode::BackTab => {
                // Move to previous field (Shift+Tab)
                self.app.metadata_input.focused_field = match self.app.metadata_input.focused_field {
                    MetadataField::ExpectedOutput => MetadataField::IsKnownFailure,
                    MetadataField::Description => MetadataField::ExpectedOutput,
                    MetadataField::UseRuntime => MetadataField::Description,
                    MetadataField::IsKnownFailure => MetadataField::UseRuntime,
                };
            }
            KeyCode::Char(' ') => {
                // Toggle checkbox fields
                match self.app.metadata_input.focused_field {
                    MetadataField::UseRuntime => {
                        self.app.metadata_input.use_runtime = !self.app.metadata_input.use_runtime;
                    }
                    MetadataField::IsKnownFailure => {
                        self.app.metadata_input.is_known_failure = !self.app.metadata_input.is_known_failure;
                    }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                // Add character to text fields
                match self.app.metadata_input.focused_field {
                    MetadataField::ExpectedOutput => {
                        // Handle special escape sequences
                        if c == '\\' && !self.app.metadata_input.expected_output.ends_with('\\') {
                            self.app.metadata_input.expected_output.push('\\');
                        } else if self.app.metadata_input.expected_output.ends_with('\\') {
                            let _ = self.app.metadata_input.expected_output.pop();
                            match c {
                                'n' => self.app.metadata_input.expected_output.push('\n'),
                                't' => self.app.metadata_input.expected_output.push('\t'),
                                'r' => self.app.metadata_input.expected_output.push('\r'),
                                '\\' => self.app.metadata_input.expected_output.push('\\'),
                                _ => {
                                    self.app.metadata_input.expected_output.push('\\');
                                    self.app.metadata_input.expected_output.push(c);
                                }
                            }
                        } else {
                            self.app.metadata_input.expected_output.push(c);
                        }
                    }
                    MetadataField::Description => {
                        self.app.metadata_input.description.push(c);
                    }
                    _ => {}
                }
            }
            KeyCode::Backspace => {
                // Remove character from text fields
                match self.app.metadata_input.focused_field {
                    MetadataField::ExpectedOutput => {
                        self.app.metadata_input.expected_output.pop();
                    }
                    MetadataField::Description => {
                        self.app.metadata_input.description.pop();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(true)
    }

    fn run_selected_test(&mut self) -> Result<()> {
        if let Some(test_name) = self.app.get_selected_test_name() {
            self.app.clear_output();
            self.app.append_output(&format!("Running test: {}\n", test_name));
            self.app.append_output(&("-".repeat(60)) );
            self.app.append_output("\n");
            
            // Auto-switch to Output tab when running a test
            self.app.selected_tab = 3;
            
            self.app.running_test = Some(test_name.clone());
            self.app.mode = AppMode::Running;
            // Don't switch focus - stay on test list for quick navigation

            let start = Instant::now();
            
            // Get test details
            let test = self.app.get_selected_test_details();
            
            // Build and run the test
            match self.compile_and_run_test(&test_name, test.as_ref()) {
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

                    self.app.append_output(&format!("\nOutput:\n{}\n", output));
                    self.app.append_output(&format!("\n{}\n", "-".repeat(60)));
                    
                    if passed {
                        self.app.append_output(&format!("✓ Test PASSED in {}ms\n", duration));
                    } else {
                        self.app.append_output(&format!("✗ Test FAILED in {}ms\n", duration));
                        if let Some(test) = test {
                            if let Some(expected) = &test.expected {
                                self.app.append_output(&format!("\nExpected:\n{}\n", expected));
                            }
                        }
                    }

                    self.app.record_test_result(test_name, result);
                }
                Err(e) => {
                    self.app.append_output(&format!("\n✗ Error: {}\n", e));
                    
                    let result = TestResult {
                        passed: false,
                        output: format!("Error: {}", e),
                        expected: test.as_ref().and_then(|t| t.expected.clone()),
                        duration_ms: start.elapsed().as_millis(),
                    };
                    
                    self.app.record_test_result(test_name, result);
                }
            }

            self.app.mode = AppMode::Normal;
        }
        Ok(())
    }

    fn debug_selected_test<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        if let Some(test_name) = self.app.get_selected_test_name() {
            if let Some(test_path) = self.app.get_selected_test_path() {
                // Temporarily exit TUI to run debugger
                self.app.append_output(&format!("\nLaunching debugger for: {}\n", test_name));
                self.app.append_output("(TUI will resume after debugger exits)\n");
                
                // We need to compile first
                let actual_test_path = if test_path.is_relative() && !test_path.starts_with("c-test") {
                    Path::new("c-test").join(&test_path)
                } else {
                    test_path.clone()
                };

                // Compile the test
                let basename = test_name;
                let asm_file = self.app.tools.build_dir.join(format!("{}.asm", basename));
                let ir_file = self.app.tools.build_dir.join(format!("{}.ir", basename));
                let pobj_file = self.app.tools.build_dir.join(format!("{}.pobj", basename));
                let bin_file = self.app.tools.build_dir.join(format!("{}.bin", basename));

                // Compile C to assembly
                let cmd = format!(
                    "{} compile {} -o {} --save-ir --ir-output {}",
                    self.app.tools.rcc.display(),
                    actual_test_path.display(),
                    asm_file.display(),
                    ir_file.display()
                );

                if let Err(e) = run_command_sync(&cmd, 30) {
                    self.app.append_output(&format!("Compilation failed: {}\n", e));
                    return Ok(());
                }

                // Assemble
                let cmd = format!(
                    "{} assemble {} -o {} --bank-size {} --max-immediate 65535",
                    self.app.tools.rasm.display(),
                    asm_file.display(),
                    pobj_file.display(),
                    self.app.bank_size
                );

                if let Err(e) = run_command_sync(&cmd, 30) {
                    self.app.append_output(&format!("Assembly failed: {}\n", e));
                    return Ok(());
                }

                // Link
                let cmd = format!(
                    "{} {} {} {} -f binary --bank-size {} -o {}",
                    self.app.tools.rlink.display(),
                    self.app.tools.crt0().display(),
                    self.app.tools.libruntime().display(),
                    pobj_file.display(),
                    self.app.bank_size,
                    bin_file.display()
                );

                if let Err(e) = run_command_sync(&cmd, 30) {
                    self.app.append_output(&format!("Linking failed: {}\n", e));
                    return Ok(());
                }

                // Now run with debugger - need to exit TUI temporarily
                // Properly suspend the terminal
                terminal.show_cursor()?;
                disable_raw_mode()?;
                execute!(
                    io::stdout(),
                    LeaveAlternateScreen,
                    cursor::Show,
                )?;
                io::stdout().flush()?;

                // Run debugger
                let status = std::process::Command::new(&self.app.tools.rvm)
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
                
                // Clear and force complete redraw
                terminal.clear()?;
                terminal.draw(|f| ui::draw(f, &mut self.app))?;

                if !status.success() {
                    self.app.append_output("Debugger exited with error\n");
                } else {
                    self.app.append_output("Debugger session ended\n");
                }
            }
        }
        Ok(())
    }

    fn run_category_tests(&mut self) -> Result<()> {
        // Get the current category name
        let category_name = match self.app.get_current_category_name() {
            Some(name) => name,
            None => {
                self.app.append_output("No category selected.\n");
                return Ok(());
            }
        };
        
        // Get tests in this category
        let tests_to_run = self.app.get_category_tests(&category_name);
        
        if tests_to_run.is_empty() {
            self.app.append_output(&format!("No tests in category '{}'.\n", category_name));
            return Ok(());
        }
        
        self.app.clear_output();
        self.app.append_output(&format!("Running all tests in category '{}'...\n", category_name));
        self.app.append_output(&("-".repeat(60)));
        self.app.append_output("\n");
        
        // Auto-switch to Output tab when running tests
        self.app.selected_tab = 3;
        
        self.run_test_batch(tests_to_run)
    }

    fn run_all_visible_tests(&mut self) -> Result<()> {
        self.app.clear_output();
        self.app.append_output("Running tests...\n");
        self.app.append_output(&("-".repeat(60)));
        self.app.append_output("\n");
        
        // Auto-switch to Output tab when running tests
        self.app.selected_tab = 3;
        
        let tests_to_run: Vec<_> = self.app.filtered_tests.clone();
        
        if tests_to_run.is_empty() {
            self.app.append_output("No tests to run.\n");
            return Ok(());
        }

        self.run_test_batch(tests_to_run)
    }
    
    fn run_test_batch(&mut self, tests_to_run: Vec<crate::config::TestCase>) -> Result<()> {
        use crate::runner::TestRunner;
        use crate::config::{RunConfig, Backend};
        
        // Create channel for communication
        let (tx, rx) = mpsc::channel();
        self.app.test_receiver = Some(rx);
        
        // Clone necessary data for the thread
        let tools = self.app.tools.clone();
        let bank_size = self.app.bank_size;
        let timeout_secs = self.app.timeout_secs;
        
        // Spawn thread to run tests using the EXACT SAME TestRunner as CLI
        thread::spawn(move || {
            // Create run config - EXACT SAME as CLI
            let run_config = RunConfig {
                backend: Backend::Rvm,
                timeout_secs,
                bank_size,
                verbose: false,
                no_cleanup: true,
                parallel: true,  // Use parallel execution like CLI
                debug_mode: false,
            };
            
            // Create the same TestRunner that CLI uses
            let runner = TestRunner::new(run_config, tools);
            
            // Prepare test references for batch execution
            let test_refs: Vec<&crate::config::TestCase> = tests_to_run.iter().collect();
            
            // Send a single progress message at the start
            let _ = tx.send(crate::tui::app::TestMessage::Progress(
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
            let _ = tx.send(crate::tui::app::TestMessage::BatchCompleted(all_results));
            
            // Send finished message
            let _ = tx.send(crate::tui::app::TestMessage::Finished);
        });
        
        self.app.mode = AppMode::Running;
        Ok(())
    }

    fn compile_and_run_test(&self, test_name: &str, test: Option<&crate::config::TestCase>) -> Result<String> {
        // Find test file path
        let test_path = if let Some(test) = test {
            test.file.clone()
        } else {
            // Try to find in the test config
            self.app.test_config.tests
                .iter()
                .find(|t| t.file.file_stem().and_then(|s| s.to_str()) == Some(test_name))
                .map(|t| t.file.clone())
                .ok_or_else(|| anyhow::anyhow!("Test not found"))?
        };

        let actual_test_path = if test_path.is_relative() && !test_path.starts_with("c-test") {
            Path::new("c-test").join(&test_path)
        } else {
            test_path
        };

        // Compile the test
        let basename = test_name;
        let asm_file = self.app.tools.build_dir.join(format!("{}.asm", basename));
        let ir_file = self.app.tools.build_dir.join(format!("{}.ir", basename));
        let pobj_file = self.app.tools.build_dir.join(format!("{}.pobj", basename));
        let bin_file = self.app.tools.build_dir.join(format!("{}.bin", basename));

        // Compile C to assembly
        let cmd = format!(
            "{} compile {} -o {} --save-ir --ir-output {}",
            self.app.tools.rcc.display(),
            actual_test_path.display(),
            asm_file.display(),
            ir_file.display()
        );

        let result = run_command_sync(&cmd, 30)?;
        if result.exit_code != 0 {
            return Err(anyhow::anyhow!("Compilation failed: {}", result.stderr));
        }

        // Assemble
        let cmd = format!(
            "{} assemble {} -o {} --bank-size {} --max-immediate 65535",
            self.app.tools.rasm.display(),
            asm_file.display(),
            pobj_file.display(),
            self.app.bank_size
        );

        let result = run_command_sync(&cmd, 30)?;
        if result.exit_code != 0 {
            return Err(anyhow::anyhow!("Assembly failed: {}", result.stderr));
        }

        // Link (with runtime if needed)
        let use_runtime = test.map(|t| t.use_runtime).unwrap_or(true);
        let cmd = if use_runtime {
            format!(
                "{} {} {} {} -f binary --bank-size {} -o {}",
                self.app.tools.rlink.display(),
                self.app.tools.crt0().display(),
                self.app.tools.libruntime().display(),
                pobj_file.display(),
                self.app.bank_size,
                bin_file.display()
            )
        } else {
            format!(
                "{} {} -f binary --bank-size {} -o {}",
                self.app.tools.rlink.display(),
                pobj_file.display(),
                self.app.bank_size,
                bin_file.display()
            )
        };

        let result = run_command_sync(&cmd, 30)?;
        if result.exit_code != 0 {
            return Err(anyhow::anyhow!("Linking failed: {}", result.stderr));
        }

        // Run the test
        let cmd = format!("{} {}", self.app.tools.rvm.display(), bin_file.display());
        let result = run_command_sync(&cmd, self.app.timeout_secs)?;
        
        if result.exit_code != 0 && result.stdout.is_empty() {
            return Err(anyhow::anyhow!("Execution failed: {}", result.stderr));
        }

        Ok(result.stdout)
    }
}