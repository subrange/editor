use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use crate::config::{TestConfig, TestCase, KnownFailure};
use crate::compiler::ToolPaths;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    TestList,
    RightPanel,  // Contains all tabs: Source, ASM, IR, Output, Details
    Filter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Filter,
    Running,
    SelectCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestCategory {
    All,
    Core,
    Advanced,
    Memory,
    Integration,
    Runtime,
    Experimental,
    KnownFailures,
    Examples,
}

impl TestCategory {
    pub fn all() -> Vec<Self> {
        vec![
            Self::All,
            Self::Core,
            Self::Advanced,
            Self::Memory,
            Self::Integration,
            Self::Runtime,
            Self::Experimental,
            Self::KnownFailures,
            Self::Examples,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            Self::All => "All Tests",
            Self::Core => "Core",
            Self::Advanced => "Advanced",
            Self::Memory => "Memory",
            Self::Integration => "Integration",
            Self::Runtime => "Runtime",
            Self::Experimental => "Experimental",
            Self::KnownFailures => "Known Failures",
            Self::Examples => "Examples",
        }
    }

    pub fn matches_test(&self, test: &TestCase) -> bool {
        if *self == TestCategory::All {
            return true;
        }

        let path_str = test.file.to_string_lossy();
        match self {
            Self::Core => path_str.contains("/core/"),
            Self::Advanced => path_str.contains("/advanced/"),
            Self::Memory => path_str.contains("/memory/"),
            Self::Integration => path_str.contains("/integration/"),
            Self::Runtime => path_str.contains("/runtime/") || path_str.contains("tests-runtime/"),
            Self::Experimental => path_str.contains("/experimental/"),
            Self::KnownFailures | Self::Examples => false,
            Self::All => true,
        }
    }

    pub fn matches_known_failure(&self, _failure: &KnownFailure) -> bool {
        matches!(self, Self::All | Self::KnownFailures)
    }

    pub fn matches_example(&self, path: &PathBuf) -> bool {
        if *self == TestCategory::All || *self == TestCategory::Examples {
            path.to_string_lossy().contains("/examples/")
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub passed: bool,
    pub output: String,
    pub expected: Option<String>,
    pub duration_ms: u128,
}

#[derive(Debug, Clone)]
pub enum TestMessage {
    Started(String),
    Completed(String, TestResult),
    BatchCompleted(Vec<(String, TestResult)>),
    Progress(String),
    Finished,
}

pub struct TuiApp {
    // UI State
    pub focused_pane: FocusedPane,
    pub mode: AppMode,
    pub selected_tab: usize,  // 0=Source, 1=ASM, 2=IR, 3=Output, 4=Details

    // Test data
    pub test_config: TestConfig,
    pub tools: ToolPaths,
    pub filtered_tests: Vec<TestCase>,
    pub filtered_failures: Vec<KnownFailure>,

    // Selection and scrolling
    pub selected_test: usize,
    pub test_scroll: usize,
    pub output_scroll: usize,
    pub source_scroll: usize,
    pub asm_scroll: usize,
    pub ir_scroll: usize,
    pub details_scroll: usize,
    pub category_scroll: usize,

    // Categories and filtering
    pub selected_category: TestCategory,
    pub filter_text: String,
    pub show_categories: bool,

    // Test execution
    pub test_results: HashMap<String, TestResult>,
    pub running_test: Option<String>,
    pub output_buffer: String,
    
    // Batch execution progress
    pub test_receiver: Option<mpsc::Receiver<TestMessage>>,
    pub tests_total: usize,
    pub tests_completed: usize,

    // Settings
    pub bank_size: usize,
    pub timeout_secs: u64,
    pub show_help: bool,
}

impl TuiApp {
    pub fn new(test_config: TestConfig, tools: ToolPaths, bank_size: usize, timeout_secs: u64) -> Self {
        let mut app = Self {
            focused_pane: FocusedPane::TestList,
            mode: AppMode::Normal,
            selected_tab: 0,

            filtered_tests: test_config.tests.clone(),
            filtered_failures: test_config.known_failures.clone(),
            test_config,
            tools,

            selected_test: 0,
            test_scroll: 0,
            output_scroll: 0,
            source_scroll: 0,
            asm_scroll: 0,
            ir_scroll: 0,
            details_scroll: 0,
            category_scroll: 0,

            selected_category: TestCategory::All,
            filter_text: String::new(),
            show_categories: false,

            test_results: HashMap::new(),
            running_test: None,
            output_buffer: String::new(),
            test_receiver: None,
            tests_total: 0,
            tests_completed: 0,

            bank_size,
            timeout_secs,
            show_help: false,
        };

        app.apply_filters();
        app
    }

    pub fn apply_filters(&mut self) {
        // Filter by category
        self.filtered_tests = self.test_config.tests
            .iter()
            .filter(|test| self.selected_category.matches_test(test))
            .cloned()
            .collect();

        self.filtered_failures = if matches!(self.selected_category, TestCategory::KnownFailures | TestCategory::All) {
            self.test_config.known_failures.clone()
        } else {
            Vec::new()
        };

        // Apply text filter if present
        if !self.filter_text.is_empty() {
            let filter_lower = self.filter_text.to_lowercase();
            
            self.filtered_tests.retain(|test| {
                test.file.to_string_lossy().to_lowercase().contains(&filter_lower) ||
                test.description.as_ref().map(|d| d.to_lowercase().contains(&filter_lower)).unwrap_or(false)
            });

            self.filtered_failures.retain(|failure| {
                failure.file.to_string_lossy().to_lowercase().contains(&filter_lower) ||
                failure.description.as_ref().map(|d| d.to_lowercase().contains(&filter_lower)).unwrap_or(false)
            });
        }

        // Reset selection if out of bounds
        let total_items = self.filtered_tests.len() + self.filtered_failures.len();
        if self.selected_test >= total_items && total_items > 0 {
            self.selected_test = total_items - 1;
        }
    }

    pub fn get_selected_test_name(&self) -> Option<String> {
        if self.selected_test < self.filtered_tests.len() {
            self.filtered_tests.get(self.selected_test)
                .and_then(|t| t.file.file_stem())
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        } else {
            let failure_idx = self.selected_test - self.filtered_tests.len();
            self.filtered_failures.get(failure_idx)
                .and_then(|f| f.file.file_stem())
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        }
    }

    pub fn get_selected_test_path(&self) -> Option<PathBuf> {
        if self.selected_test < self.filtered_tests.len() {
            self.filtered_tests.get(self.selected_test)
                .map(|t| t.file.clone())
        } else {
            let failure_idx = self.selected_test - self.filtered_tests.len();
            self.filtered_failures.get(failure_idx)
                .map(|f| f.file.clone())
        }
    }

    pub fn get_selected_test_details(&self) -> Option<TestCase> {
        if self.selected_test < self.filtered_tests.len() {
            self.filtered_tests.get(self.selected_test).cloned()
        } else {
            let failure_idx = self.selected_test - self.filtered_tests.len();
            self.filtered_failures.get(failure_idx).map(|f| TestCase {
                file: f.file.clone(),
                expected: None,
                use_runtime: true,
                description: f.description.clone(),
            })
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_test > 0 {
            self.selected_test -= 1;
            self.ensure_selection_visible();
        }
    }

    pub fn move_selection_down(&mut self) {
        let total_items = self.filtered_tests.len() + self.filtered_failures.len();
        if self.selected_test < total_items.saturating_sub(1) {
            self.selected_test += 1;
            self.ensure_selection_visible();
        }
    }

    pub fn ensure_selection_visible(&mut self) {
        // Adjust scroll to keep selection visible
        const VISIBLE_ITEMS: usize = 20; // Approximate visible items in list
        
        if self.selected_test < self.test_scroll {
            self.test_scroll = self.selected_test;
        } else if self.selected_test >= self.test_scroll + VISIBLE_ITEMS {
            self.test_scroll = self.selected_test.saturating_sub(VISIBLE_ITEMS - 1);
        }
    }

    pub fn toggle_category_selection(&mut self) {
        self.show_categories = !self.show_categories;
        if self.show_categories {
            self.mode = AppMode::SelectCategory;
        } else {
            self.mode = AppMode::Normal;
        }
    }

    pub fn select_category(&mut self, category: TestCategory) {
        self.selected_category = category;
        self.apply_filters();
        self.show_categories = false;
        self.mode = AppMode::Normal;
    }

    pub fn start_filter(&mut self) {
        self.mode = AppMode::Filter;
        self.focused_pane = FocusedPane::Filter;
    }

    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.apply_filters();
        self.mode = AppMode::Normal;
        self.focused_pane = FocusedPane::TestList;
    }

    pub fn append_output(&mut self, text: &str) {
        self.output_buffer.push_str(text);
        // Don't auto-scroll, let user control scrolling
    }

    pub fn clear_output(&mut self) {
        self.output_buffer.clear();
        self.output_scroll = 0;
    }

    pub fn record_test_result(&mut self, test_name: String, result: TestResult) {
        self.test_results.insert(test_name, result);
        self.running_test = None;
    }
}