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
    
    pub fn jump_to_test(&mut self, test_name: &str) {
        self.app.jump_to_test_by_name(test_name);
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
                self.app.append_output(&format!("Failed to build runtime: {e}\n"));
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
            println!("{err:?}");
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
                self.app.append_output(&format!("Running: {test_name}\n"));
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
                // Simple version - just process results without fancy output
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
                
                // Show simple summary
                self.app.append_output(&format!("\n{}\n", "=".repeat(60)));
                self.app.append_output(&format!("Results: {} passed, {} failed, {} total\n", 
                    passed, failed, passed + failed));
            }
            TestMessage::Progress(msg) => {
                self.app.append_output(&format!("{msg}\n"));
                
                // Auto-scroll to show the latest progress
                let total_lines = self.app.output_buffer.lines().count();
                let visible_lines = 20; // Approximate visible lines
                if total_lines > visible_lines {
                    self.app.output_scroll = total_lines.saturating_sub(visible_lines);
                }
            }
            TestMessage::Finished => {
                self.app.test_receiver = None;
                self.app.mode = AppMode::Normal;
                
                // Ensure we're in a valid state
                if self.app.output_buffer.contains("ERROR:") {
                    self.app.append_output("\n⚠️  Test execution encountered errors. Check output above.\n");
                }
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
            AppMode::EditingExpected => self.handle_edit_expected_input(key),
            AppMode::RenamingTest => self.handle_rename_test_input(key),
            AppMode::MovingTest => self.handle_move_test_input(key),
            AppMode::CreatingTest => self.handle_create_test_input(key),
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
                        if let Err(e) = self.run_selected_test() {
                            self.app.append_output(&format!("Error running test: {e}\n"));
                        }
                    }
                    SelectedItemType::None => {}
                }
            }
            KeyCode::Char(' ') => {
                // Space also toggles category expansion
                self.app.toggle_current_category();
            }
            KeyCode::Char('d') => {
                if let Err(e) = self.debug_selected_test(terminal) {
                    self.app.append_output(&format!("Error debugging test: {e}\n"));
                }
            }
            KeyCode::Char('r') => {
                if let Err(e) = self.run_all_visible_tests() {
                    self.app.append_output(&format!("Error running tests: {e}\n"));
                }
            }
            KeyCode::Char('R') => {
                // Shift+R - run all tests in current category
                if let Err(e) = self.run_category_tests() {
                    self.app.append_output(&format!("Error running category tests: {e}\n"));
                }
            }
            KeyCode::Char('c') => {
                self.app.toggle_category_selection();
            }
            KeyCode::Char('/') => {
                self.app.start_find_test();
            }
            KeyCode::Char('a') => {
                // Add new test from template
                self.app.start_create_test();
            }
            KeyCode::Char('A') => {
                // Quick add orphan test metadata with current output (Shift+A)
                if self.app.is_current_test_orphan() {
                    // First run the test to get output if needed
                    let test_name = self.app.get_selected_test_name();
                    if let Some(name) = test_name {
                        if !self.app.test_results.contains_key(&name) {
                            // Run the test first to get output
                            self.run_selected_test()?;
                        }
                        // Now add the metadata with the output
                        if let Err(e) = self.app.quick_add_orphan_metadata() {
                            self.app.append_output(&format!("Failed to add metadata: {e}\n"));
                        }
                    }
                } else {
                    self.app.append_output("Current test is not an orphan test.\n");
                }
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
            KeyCode::Char('e') => {
                // Edit selected test in vim
                self.edit_selected_test(terminal)?;
            }
            KeyCode::Char('E') => {
                // Edit expected output (Shift+E)
                self.app.start_edit_expected();
            }
            KeyCode::Char('g') => {
                // Golden update - apply actual output as expected for failing test
                if let Err(e) = self.app.apply_golden_output() {
                    self.app.append_output(&format!("Failed to apply golden output: {e}\n"));
                }
            }
            KeyCode::Char('G') => {
                // Shift+G - Jump to end in output view
                if self.app.focused_pane == FocusedPane::RightPanel && self.app.selected_tab == 3 {
                    let total_lines = self.app.output_buffer.lines().count();
                    let visible_lines = 20; // Approximate visible lines
                    if total_lines > visible_lines {
                        self.app.output_scroll = total_lines.saturating_sub(visible_lines);
                    }
                } else if self.app.focused_pane == FocusedPane::TestList {
                    // In test list, go to last test
                    let total_items = self.app.get_total_visible_items();
                    if total_items > 0 {
                        self.app.selected_item = total_items - 1;
                        self.app.ensure_selection_visible();
                    }
                }
            }
            KeyCode::Char('n') => {
                // Rename selected test
                self.app.start_rename_test();
            }
            KeyCode::Char('s') => {
                // Toggle skip status for selected test
                if let Err(e) = self.app.toggle_skip_status() {
                    self.app.append_output(&format!("Failed to toggle skip status: {e}\n"));
                } else {
                    self.app.append_output("Skip status toggled successfully!\n");
                }
            }
            KeyCode::Char('M') => {
                // Move selected test to different category (Shift+M)
                self.app.start_move_test();
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
            KeyCode::F(5) => {
                // Force complete UI refresh and reload tests
                
                // Clear the terminal completely
                terminal.clear()?;
                
                // Reset all scroll positions
                self.app.source_scroll = 0;
                self.app.asm_scroll = 0;
                self.app.ir_scroll = 0;
                self.app.output_scroll = 0;
                self.app.details_scroll = 0;
                self.app.help_scroll = 0;
                
                // Clear output buffer to start fresh
                self.app.clear_output();
                
                // Reload all tests from filesystem
                self.app.reload_all_tests();
                
                // Force immediate redraw
                terminal.draw(|f| ui::draw(f, &mut self.app))?;
                
                self.app.append_output("UI refreshed and tests reloaded.\n");
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
                        if self.app.selected_tab == 3 {
                            let total_lines = self.app.output_buffer.lines().count();
                            self.app.output_scroll = total_lines.saturating_sub(10);
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
                    self.app.append_output(&format!("Failed to delete test: {e}\n"));
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

    fn handle_edit_expected_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                // Cancel editing
                self.app.cancel_edit_expected();
            }
            KeyCode::Enter => {
                // Save expected output
                if let Err(e) = self.app.save_expected_output() {
                    self.app.append_output(&format!("Failed to save expected output: {e}\n"));
                }
            }
            KeyCode::Char(c) => {
                // Handle special escape sequences
                if c == '\\' && !self.app.editing_expected.ends_with('\\') {
                    self.app.editing_expected.push('\\');
                } else if self.app.editing_expected.ends_with('\\') {
                    let _ = self.app.editing_expected.pop();
                    match c {
                        'n' => self.app.editing_expected.push('\n'),
                        't' => self.app.editing_expected.push('\t'),
                        'r' => self.app.editing_expected.push('\r'),
                        '\\' => self.app.editing_expected.push('\\'),
                        _ => {
                            self.app.editing_expected.push('\\');
                            self.app.editing_expected.push(c);
                        }
                    }
                } else {
                    self.app.editing_expected.push(c);
                }
            }
            KeyCode::Backspace => {
                self.app.editing_expected.pop();
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
                    self.app.append_output(&format!("Failed to save metadata: {e}\n"));
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
            self.app.append_output(&format!("Running test: {test_name}\n"));
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

                    self.app.append_output(&format!("\nOutput:\n{output}\n"));
                    self.app.append_output(&format!("\n{}\n", "-".repeat(60)));
                    
                    if passed {
                        self.app.append_output(&format!("✓ Test PASSED in {duration}ms\n"));
                    } else {
                        self.app.append_output(&format!("✗ Test FAILED in {duration}ms\n"));
                        if let Some(test) = test {
                            if let Some(expected) = &test.expected {
                                self.app.append_output(&format!("\nExpected:\n{expected}\n"));
                            }
                        }
                    }

                    self.app.record_test_result(test_name, result);
                }
                Err(e) => {
                    self.app.append_output(&format!("\n✗ Error: {e}\n"));
                    
                    let result = TestResult {
                        passed: false,
                        output: format!("Error: {e}"),
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

    fn edit_selected_test<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        if let Some(test_path) = self.app.get_selected_test_path_for_edit() {
            let test_name = test_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            
            // Notify user
            self.app.append_output(&format!("\nOpening {test_name} in vim...\n"));
            self.app.append_output("(TUI will resume after vim exits)\n");
            
            // Pause the event handler FIRST
            self.events.pause();
            
            // Properly suspend the terminal
            terminal.show_cursor()?;
            disable_raw_mode()?;
            execute!(
                io::stdout(),
                LeaveAlternateScreen,
                cursor::Show,
            )?;
            io::stdout().flush()?;
            
            // Open vim to edit the file
            let status = std::process::Command::new("vim")
                .arg(&test_path)
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
            while self.events.rx.try_recv().is_ok() {
                // Discard any queued events
            }
            
            // Resume the event handler
            self.events.resume();
            
            // Clear and force complete redraw
            terminal.clear()?;
            
            // Refresh the test content in the app
            self.app.refresh_test_content();
            
            // Force redraw with refreshed content
            terminal.draw(|f| ui::draw(f, &mut self.app))?;
            
            if !status.success() {
                self.app.append_output("Vim exited with error\n");
            } else {
                self.app.append_output(&format!("Finished editing {test_name}\n"));
            }
        } else {
            self.app.append_output("No test selected for editing.\n");
        }
        Ok(())
    }

    fn debug_selected_test<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        if let Some(test_name) = self.app.get_selected_test_name() {
            if let Some(test_path) = self.app.get_selected_test_path() {
                // Temporarily exit TUI to run debugger
                self.app.append_output(&format!("\nLaunching debugger for: {test_name}\n"));
                self.app.append_output("(TUI will resume after debugger exits)\n");
                
                // We need to compile first
                let actual_test_path = if test_path.is_relative() && !test_path.starts_with("c-test") {
                    Path::new("c-test").join(&test_path)
                } else {
                    test_path.clone()
                };

                // Use the proper compile_c_file function to handle compilation
                use crate::compiler::compile_c_file;
                use crate::config::{RunConfig, Backend as CompilerBackend};
                
                let run_config = RunConfig {
                    backend: CompilerBackend::Rvm,
                    timeout_secs: 30,
                    bank_size: self.app.bank_size,
                    verbose: false,
                    no_cleanup: true,
                    parallel: false,
                    debug_mode: false,
                    frequency: None,
                };
                
                // Get test details to determine if runtime is needed
                let test = self.app.get_selected_test_details();
                let use_runtime = test.as_ref().map(|t| t.use_runtime).unwrap_or(true);
                
                // Compile the test using the same function as the test runner
                let compilation_result = compile_c_file(
                    &actual_test_path,
                    &self.app.tools,
                    &run_config,
                    use_runtime,
                );
                
                match compilation_result {
                    Ok(result) if !result.success => {
                        self.app.append_output(&format!("Compilation failed: {}\n", 
                            result.error_message.unwrap_or_else(|| "Unknown error".to_string())));
                        return Ok(());
                    }
                    Err(e) => {
                        self.app.append_output(&format!("Compilation error: {e}\n"));
                        return Ok(());
                    }
                    _ => {} // Success, continue
                }
                
                // The binary should now exist at the expected location
                let bin_file = self.app.tools.build_dir.join(format!("{test_name}.bin"));

                // Now run with debugger - need to exit TUI temporarily
                // Pause the event handler FIRST
                self.events.pause();
                
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
                
                // Clear any events that might have been queued
                while self.events.rx.try_recv().is_ok() {
                    // Discard any queued events
                }
                
                // Resume the event handler
                self.events.resume();
                
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
            self.app.append_output(&format!("No tests in category '{category_name}'.\n"));
            return Ok(());
        }
        
        self.app.clear_output();
        let test_count = tests_to_run.len();
        self.app.append_output(&format!("📁 Running {} test{} in category '{}'...\n", 
            test_count,
            if test_count == 1 { "" } else { "s" },
            category_name
        ));
        self.app.append_output(&("=".repeat(60)));
        self.app.append_output("\n\n");
        
        // Auto-switch to Output tab when running tests
        self.app.selected_tab = 3;
        // Reset scroll to top to see the start of the test run
        self.app.output_scroll = 0;
        
        self.run_test_batch(tests_to_run)
    }

    fn run_all_visible_tests(&mut self) -> Result<()> {
        self.app.clear_output();
        
        let tests_to_run: Vec<_> = self.app.filtered_tests.clone();
        
        if tests_to_run.is_empty() {
            self.app.append_output("No tests to run.\n");
            return Ok(());
        }
        
        let test_count = tests_to_run.len();
        self.app.append_output(&format!("🚀 Running {} test{}...\n", 
            test_count, 
            if test_count == 1 { "" } else { "s" }
        ));
        self.app.append_output(&("=".repeat(60)));
        self.app.append_output("\n\n");
        
        // Auto-switch to Output tab when running tests
        self.app.selected_tab = 3;
        // Reset scroll to top to see the start of the test run
        self.app.output_scroll = 0;

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
            }));
            
            // If the test runner panicked, send an error message
            if result.is_err() {
                let _ = tx_panic.send(crate::tui::app::TestMessage::Progress(
                    "ERROR: Test runner crashed unexpectedly".to_string()
                ));
                let _ = tx_panic.send(crate::tui::app::TestMessage::Finished);
            }
        });
        
        self.app.mode = AppMode::Running;
        Ok(())
    }

    fn compile_and_run_test(&self, test_name: &str, test: Option<&crate::config::TestCase>) -> Result<String> {
        use crate::compiler::compile_c_file;
        use crate::config::{RunConfig, Backend};
        
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
            test_path.clone()
        };

        // Use the same compile_c_file function that the main test runner uses
        // This properly handles preprocessing and compiler errors/panics
        let run_config = RunConfig {
            backend: Backend::Rvm,
            timeout_secs: self.app.timeout_secs,
            bank_size: self.app.bank_size,
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
            &self.app.tools,
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
    
    fn handle_rename_test_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                // Cancel renaming
                self.app.cancel_rename();
            }
            KeyCode::Enter => {
                // Save new name
                if let Err(e) = self.app.save_rename_test() {
                    self.app.append_output(&format!("Failed to rename test: {e}\n"));
                }
            }
            KeyCode::Char(c) => {
                // Only allow alphanumeric, underscore, and hyphen in test names
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    self.app.rename_new_name.push(c);
                }
            }
            KeyCode::Backspace => {
                self.app.rename_new_name.pop();
            }
            _ => {}
        }
        Ok(true)
    }
    
    fn handle_move_test_input(&mut self, key: KeyEvent) -> Result<bool> {
        // In move mode, we show the category selector
        // So we delegate to the category input handler
        match key.code {
            KeyCode::Esc => {
                // Cancel moving
                self.app.cancel_move();
            }
            KeyCode::Up => {
                self.app.move_category_selection_up();
            }
            KeyCode::Down => {
                self.app.move_category_selection_down();
            }
            KeyCode::Enter => {
                // Get selected category
                let target_category = if self.app.selected_category_index == 0 {
                    // "All Tests" option - use Uncategorized
                    "Uncategorized".to_string()
                } else {
                    // Get the category at the current index
                    let category_names: Vec<String> = self.app.categories.keys().cloned().collect();
                    category_names.get(self.app.selected_category_index - 1)
                        .cloned()
                        .unwrap_or_else(|| "Uncategorized".to_string())
                };
                
                // Save the move with the selected category
                if let Err(e) = self.app.save_move_test(target_category) {
                    self.app.append_output(&format!("Failed to move test: {e}\n"));
                }
            }
            _ => {}
        }
        Ok(true)
    }
    
    fn handle_create_test_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                // Cancel creating
                self.app.cancel_create_test();
            }
            KeyCode::Enter => {
                // Save new test
                if let Err(e) = self.app.save_new_test() {
                    self.app.append_output(&format!("Failed to create test: {e}\n"));
                }
            }
            KeyCode::Tab => {
                // Switch between name and description fields
                self.app.new_test_focused_field = !self.app.new_test_focused_field;
            }
            KeyCode::Char(c) => {
                if !self.app.new_test_focused_field {
                    // Editing name field - only allow alphanumeric, underscore, and hyphen
                    if c.is_alphanumeric() || c == '_' || c == '-' {
                        self.app.new_test_name.push(c);
                    }
                } else {
                    // Editing description field
                    self.app.new_test_description.push(c);
                }
            }
            KeyCode::Backspace => {
                if !self.app.new_test_focused_field {
                    self.app.new_test_name.pop();
                } else {
                    self.app.new_test_description.pop();
                }
            }
            _ => {}
        }
        Ok(true)
    }
}