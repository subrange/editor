use std::io::{self, Write};
use std::time::Duration;
use anyhow::Result;
use crossterm::{
    cursor,
    execute,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::compiler::{build_runtime, ToolPaths};
use crate::config::TestConfig;
use crate::tui::{
    app::{TuiApp, AppMode, TestMessage, FocusedPane, SelectedItemType},
    event::{Event, EventHandler, KeyEvent, MouseEvent},
    ui,
    handlers,
    executor,
};
use crossterm::event::{KeyCode, KeyModifiers};

pub struct TuiRunner {
    pub app: TuiApp,
    pub events: EventHandler,
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
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
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
            DisableMouseCapture,
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
                    // Handle help modal first
                    if !handlers::handle_input(&mut self.app, key)? {
                        return Ok(());
                    }
                    
                    // Handle normal mode input directly here
                    if self.app.mode == AppMode::Normal && !self.app.show_help {
                        if !self.handle_normal_input(key, terminal)? {
                            return Ok(());
                        }
                    }
                }
                Event::Tick => {
                    // Handle any background updates
                }
                Event::Mouse(mouse) => {
                    if self.app.mode == AppMode::Normal && !self.app.show_help {
                        self.handle_mouse_input(mouse);
                    }
                }
            }
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
                        // Shift+Down scrolls faster in right panel
                        let scroll_amount = if key.modifiers.contains(KeyModifiers::SHIFT) { 10 } else { 1 };
                        match self.app.selected_tab {
                            0 => self.app.source_scroll = self.app.source_scroll.saturating_add(scroll_amount),
                            1 => self.app.asm_scroll = self.app.asm_scroll.saturating_add(scroll_amount),
                            2 => self.app.ir_scroll = self.app.ir_scroll.saturating_add(scroll_amount),
                            3 => self.app.output_scroll = self.app.output_scroll.saturating_add(scroll_amount),
                            4 => self.app.details_scroll = self.app.details_scroll.saturating_add(scroll_amount),
                            5 => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    // Shift+Down moves down 10 times in tree
                                    for _ in 0..10 {
                                        self.app.ast_move_down();
                                    }
                                } else {
                                    self.app.ast_move_down();
                                }
                            }
                            6 => self.app.symbols_scroll = self.app.symbols_scroll.saturating_add(scroll_amount),
                            7 => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    // Shift+Down moves down 10 times in tree
                                    for _ in 0..10 {
                                        self.app.typed_ast_move_down();
                                    }
                                } else {
                                    self.app.typed_ast_move_down();
                                }
                            }
                            _ => {}
                        }
                    }
                    FocusedPane::TestList => {
                        // Shift+Down moves selection down by 10 items
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            for _ in 0..10 {
                                self.app.move_selection_down();
                            }
                        } else {
                            self.app.move_selection_down();
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        // Shift+Up scrolls faster in right panel
                        let scroll_amount = if key.modifiers.contains(KeyModifiers::SHIFT) { 10 } else { 1 };
                        match self.app.selected_tab {
                            0 => self.app.source_scroll = self.app.source_scroll.saturating_sub(scroll_amount),
                            1 => self.app.asm_scroll = self.app.asm_scroll.saturating_sub(scroll_amount),
                            2 => self.app.ir_scroll = self.app.ir_scroll.saturating_sub(scroll_amount),
                            3 => self.app.output_scroll = self.app.output_scroll.saturating_sub(scroll_amount),
                            4 => self.app.details_scroll = self.app.details_scroll.saturating_sub(scroll_amount),
                            5 => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    // Shift+Up moves up 10 times in tree
                                    for _ in 0..10 {
                                        self.app.ast_move_up();
                                    }
                                } else {
                                    self.app.ast_move_up();
                                }
                            }
                            6 => self.app.symbols_scroll = self.app.symbols_scroll.saturating_sub(scroll_amount),
                            7 => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    // Shift+Up moves up 10 times in tree
                                    for _ in 0..10 {
                                        self.app.typed_ast_move_up();
                                    }
                                } else {
                                    self.app.typed_ast_move_up();
                                }
                            }
                            _ => {}
                        }
                    }
                    FocusedPane::TestList => {
                        // Shift+Up moves selection up by 10 items  
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            for _ in 0..10 {
                                self.app.move_selection_up();
                            }
                        } else {
                            self.app.move_selection_up();
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Enter => {
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        match self.app.selected_tab {
                            5 => self.app.ast_toggle_current(),  // Toggle AST node expansion
                            7 => self.app.typed_ast_toggle_current(),  // Toggle TypedAST node expansion
                            _ => {}
                        }
                    }
                    FocusedPane::TestList => {
                        match self.app.get_selected_item_type() {
                            SelectedItemType::Category(_) => {
                                self.app.toggle_current_category();
                            }
                            SelectedItemType::Test(_) => {
                                if let Err(e) = self.run_selected_test() {
                                    self.app.append_output(&format!("Error running test: {e}\n"));
                                }
                            }
                            SelectedItemType::None => {}
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Char(' ') => {
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        match self.app.selected_tab {
                            5 => self.app.ast_toggle_current(),  // Space also toggles in tree view
                            7 => self.app.typed_ast_toggle_current(),
                            _ => {}
                        }
                    }
                    FocusedPane::TestList => {
                        self.app.toggle_current_category();
                    }
                    _ => {}
                }
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
                self.app.start_create_test();
            }
            KeyCode::Char('A') => {
                if self.app.is_current_test_orphan() {
                    let test_name = self.app.get_selected_test_name();
                    if let Some(name) = test_name {
                        if !self.app.test_results.contains_key(&name) {
                            self.run_selected_test()?;
                        }
                        if let Err(e) = self.app.quick_add_orphan_metadata() {
                            self.app.append_output(&format!("Failed to add metadata: {e}\n"));
                        }
                    }
                } else {
                    self.app.append_output("Current test is not an orphan test.\n");
                }
            }
            KeyCode::Char('m') => {
                if self.app.is_current_test_orphan() {
                    self.app.start_adding_metadata();
                }
            }
            KeyCode::Char('x') | KeyCode::Delete => {
                self.app.start_delete_test();
            }
            KeyCode::Char('o') => {
                self.app.jump_to_first_orphan();
            }
            KeyCode::Char('e') => {
                self.edit_selected_test(terminal)?;
            }
            KeyCode::Char('E') => {
                self.app.start_edit_expected();
            }
            KeyCode::Char('t') => {
                self.open_terminal_shell(terminal)?;
            }
            KeyCode::Char('h') | KeyCode::Left => {
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        match self.app.selected_tab {
                            5 => self.app.ast_collapse_current(),  // Collapse current node
                            7 => self.app.typed_ast_collapse_current(),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        match self.app.selected_tab {
                            5 => self.app.ast_expand_current(),  // Expand current node
                            7 => self.app.typed_ast_expand_current(),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Char('H') => {
                // Shift+H - collapse all nodes
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        match self.app.selected_tab {
                            5 => self.app.ast_collapse_all(),
                            7 => self.app.typed_ast_collapse_all(),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Char('L') => {
                // Shift+L - expand all nodes
                match self.app.focused_pane {
                    FocusedPane::RightPanel => {
                        match self.app.selected_tab {
                            5 => self.app.ast_expand_all(),
                            7 => self.app.typed_ast_expand_all(),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Char('O') => {
                // Shift+O - expand all categories
                if self.app.focused_pane == FocusedPane::TestList {
                    self.app.expand_all_categories();
                }
            }
            KeyCode::Char('C') => {
                // Shift+C - collapse all categories
                if self.app.focused_pane == FocusedPane::TestList {
                    self.app.collapse_all_categories();
                }
            }
            KeyCode::Char('g') => {
                if self.app.focused_pane == FocusedPane::TestList {
                    if let Err(e) = self.app.apply_golden_output() {
                        self.app.append_output(&format!("Failed to apply golden output: {e}\n"));
                    }
                }
            }
            KeyCode::Char('G') => {
                if self.app.focused_pane == FocusedPane::RightPanel && self.app.selected_tab == 3 {
                    let total_lines = self.app.output_buffer.lines().count();
                    let visible_lines = 20;
                    if total_lines > visible_lines {
                        self.app.output_scroll = total_lines.saturating_sub(visible_lines);
                    }
                } else if self.app.focused_pane == FocusedPane::TestList {
                    let total_items = self.app.get_total_visible_items();
                    if total_items > 0 {
                        self.app.selected_item = total_items - 1;
                        self.app.ensure_selection_visible();
                    }
                }
            }
            KeyCode::Char('n') => {
                self.app.start_rename_test();
            }
            KeyCode::Char('s') => {
                if let Err(e) = self.app.toggle_skip_status() {
                    self.app.append_output(&format!("Failed to toggle skip status: {e}\n"));
                } else {
                    self.app.append_output("Skip status toggled successfully!\n");
                }
            }
            KeyCode::Char('M') => {
                self.app.start_move_test();
            }
            KeyCode::Tab => {
                self.app.focused_pane = match self.app.focused_pane {
                    FocusedPane::TestList => FocusedPane::RightPanel,
                    FocusedPane::RightPanel => FocusedPane::TestList,
                    FocusedPane::Filter => FocusedPane::TestList,
                };
            }
            KeyCode::Char('1') => self.app.selected_tab = 0,
            KeyCode::Char('2') => self.app.selected_tab = 1,
            KeyCode::Char('3') => self.app.selected_tab = 2,
            KeyCode::Char('4') => self.app.selected_tab = 3,
            KeyCode::Char('5') => self.app.selected_tab = 4,
            KeyCode::Char('6') => self.app.selected_tab = 5,
            KeyCode::Char('7') => self.app.selected_tab = 6,
            KeyCode::Char('8') => self.app.selected_tab = 7,
            KeyCode::F(5) => {
                terminal.clear()?;
                self.app.source_scroll = 0;
                self.app.asm_scroll = 0;
                self.app.ir_scroll = 0;
                self.app.output_scroll = 0;
                self.app.details_scroll = 0;
                self.app.help_scroll = 0;
                self.app.ast_scroll = 0;
                self.app.symbols_scroll = 0;
                self.app.typed_ast_scroll = 0;
                self.app.clear_output();
                self.app.reload_all_tests();
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
                            5 => self.app.ast_scroll = self.app.ast_scroll.saturating_add(20),
                            6 => self.app.symbols_scroll = self.app.symbols_scroll.saturating_add(20),
                            7 => self.app.typed_ast_scroll = self.app.typed_ast_scroll.saturating_add(20),
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
                            5 => self.app.ast_scroll = self.app.ast_scroll.saturating_sub(20),
                            6 => self.app.symbols_scroll = self.app.symbols_scroll.saturating_sub(20),
                            7 => self.app.typed_ast_scroll = self.app.typed_ast_scroll.saturating_sub(20),
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
                            5 => self.app.ast_scroll = 0,
                            6 => self.app.symbols_scroll = 0,
                            7 => self.app.typed_ast_scroll = 0,
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
    
    fn handle_mouse_input(&mut self, mouse: MouseEvent) {
        use crossterm::event::MouseEventKind;
        
        // Only handle single left button clicks (not double clicks or drags)
        match mouse.kind {
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            // Get terminal size to properly calculate panel boundaries
            let terminal_size = crossterm::terminal::size().unwrap_or((150, 50));
            let terminal_width = terminal_size.0;
            
            // Test list takes 35% of width (matching the layout in ui/layout.rs)
            // We calculate 35% and subtract 1 for the border between panels
            let test_list_width = (terminal_width * 35 / 100).saturating_sub(1) as u16;
            
            if mouse.column <= test_list_width {
                // Calculate which item was clicked based on row
                // Account for borders and title (1 row for top border + title)
                if mouse.row > 0 {
                    let clicked_index = (mouse.row - 1) as usize + self.app.test_scroll;
                    self.app.handle_test_list_click(clicked_index);
                }
            } else {
                // Click is in the right panel
                
                // Check if click is on the tab bar (row 1, since row 0 is the border)
                if mouse.row == 1 {
                    // Tab bar clicked - calculate which tab based on approximate positions
                    // Tab titles: ["Source", "ASM", "IR", "Output", "Details", "AST", "Symbols", "TypedAST"]
                    // Calculate relative position from the start of the right panel
                    let right_panel_start = (terminal_width * 35 / 100) as u16;
                    
                    if mouse.column > right_panel_start {
                        let relative_x = mouse.column - right_panel_start;
                        
                        // Approximate tab positions (each tab name + spacing)
                        // These are rough estimates based on tab text lengths
                        let tab_boundaries = [
                            0,   // Source starts at 0
                            10,  // ASM starts around 10
                            16,  // IR starts around 16  
                            20,  // Output starts around 20
                            29,  // Details starts around 29
                            39,  // AST starts around 39
                            45,  // Symbols starts around 45
                            55,  // TypedAST starts around 55
                        ];
                        
                        // Find which tab was clicked
                        let mut clicked_tab = 7; // Default to last tab
                        for (i, &boundary) in tab_boundaries.iter().enumerate().rev() {
                            if relative_x >= boundary {
                                clicked_tab = i;
                                break;
                            }
                        }
                        
                        self.app.selected_tab = clicked_tab;
                    }
                } else {
                    // Check if it's in AST or TypedAST tab content area
                    match self.app.selected_tab {
                        5 => {
                            // AST tab - handle tree click
                            // Account for tab header (3 rows for tabs + border)
                            if mouse.row > 3 {
                                let clicked_line = (mouse.row - 4) as usize + self.app.ast_scroll;
                                self.app.handle_ast_click(clicked_line);
                            }
                        }
                        7 => {
                            // TypedAST tab - handle tree click
                            // Account for tab header (3 rows for tabs + border)
                            if mouse.row > 3 {
                                let clicked_line = (mouse.row - 4) as usize + self.app.typed_ast_scroll;
                                self.app.handle_typed_ast_click(clicked_line);
                            }
                        }
                        _ => {}
                    }
                }
            }
            }
            _ => {
                // Ignore other mouse events (move, release, drag, etc.)
            }
        }
    }
    
    fn handle_test_message(&mut self, msg: TestMessage) {
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

    // Public methods called from handlers
    pub fn run_selected_test(&mut self) -> Result<()> {
        if let Some(test_name) = self.app.get_selected_test_name() {
            executor::run_single_test(&mut self.app, &test_name)
        } else {
            Ok(())
        }
    }

    pub fn run_all_visible_tests(&mut self) -> Result<()> {
        let tests_to_run = self.app.filtered_tests.clone();
        executor::run_batch_tests(&mut self.app, tests_to_run)
    }

    pub fn run_category_tests(&mut self) -> Result<()> {
        // Get the current category name
        let category_name = match self.app.get_current_category_name() {
            Some(name) => name,
            None => {
                self.app.append_output("No category selected.\n");
                return Ok(());
            }
        };
        
        executor::run_category_tests(&mut self.app, &category_name)
    }

    pub fn debug_selected_test<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        executor::debug_test(&mut self.app, terminal, &self.events)
    }

    pub fn edit_selected_test<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
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
                DisableMouseCapture,
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
                EnableMouseCapture,
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

    pub fn open_terminal_shell<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // Notify user
        self.app.append_output("\nOpening terminal shell...\n");
        self.app.append_output("Type 'exit' to return to the test runner.\n");
        
        // Pause the event handler FIRST
        self.events.pause();
        
        // Properly suspend the terminal
        terminal.show_cursor()?;
        disable_raw_mode()?;
        execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show,
        )?;
        io::stdout().flush()?;
        
        // Open the user's shell (try $SHELL first, fallback to sh)
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let status = std::process::Command::new(&shell)
            .status()?;
        
        // Properly restore the terminal
        // Small delay to ensure terminal processes the mode change
        std::thread::sleep(Duration::from_millis(100));
        
        enable_raw_mode()?;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
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
        
        // Force redraw
        terminal.draw(|f| ui::draw(f, &mut self.app))?;
        
        if !status.success() {
            self.app.append_output("Shell exited with error\n");
        } else {
            self.app.append_output("Returned from terminal shell\n");
        }
        
        Ok(())
    }
}