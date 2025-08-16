use std::collections::{HashMap, BTreeMap};
use std::path::PathBuf;
use std::sync::mpsc;
use crate::config::{TestConfig, TestCase, KnownFailure};
use crate::compiler::ToolPaths;

#[derive(Debug, Clone)]
pub enum SelectedItemType {
    None,
    Category(String),
    Test(TestCase),
}

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

#[derive(Debug, Clone)]
pub struct CategoryView {
    pub name: String,
    pub tests: Vec<TestCase>,
    pub expanded: bool,
    pub test_count: usize,
}

impl CategoryView {
    pub fn from_tests(tests: &[TestCase], failures: &[KnownFailure]) -> BTreeMap<String, CategoryView> {
        let mut categories: BTreeMap<String, Vec<TestCase>> = BTreeMap::new();
        
        // Group tests by category
        for test in tests {
            let category = Self::get_category_from_path(&test.file);
            categories.entry(category).or_insert_with(Vec::new).push(test.clone());
        }
        
        // Add known failures as a category
        if !failures.is_empty() {
            let mut failure_tests = Vec::new();
            for failure in failures {
                failure_tests.push(TestCase {
                    file: failure.file.clone(),
                    expected: None,
                    use_runtime: true,
                    description: failure.description.clone(),
                });
            }
            categories.insert("Known Failures".to_string(), failure_tests);
        }
        
        // Convert to CategoryView
        let mut result = BTreeMap::new();
        for (name, tests) in categories {
            result.insert(name.clone(), CategoryView {
                name: name.clone(),
                test_count: tests.len(),
                tests,
                expanded: true, // Start expanded
            });
        }
        
        result
    }
    
    fn get_category_from_path(path: &PathBuf) -> String {
        let path_str = path.to_string_lossy();
        let parts: Vec<&str> = path_str.split('/').collect();
        
        // Find where test directories start
        let mut start_idx = None;
        for (i, part) in parts.iter().enumerate() {
            if *part == "tests" || *part == "tests-runtime" || 
               *part == "known-failures" || *part == "examples" {
                start_idx = Some(i);
                break;
            }
        }
        
        if let Some(idx) = start_idx {
            let root = parts[idx];
            
            match root {
                "tests-runtime" => "Runtime".to_string(),
                "examples" => "Examples".to_string(),
                "known-failures" => {
                    // Check for subdirectory
                    if parts.len() > idx + 2 {
                        format!("Known Failures › {}", Self::capitalize_words(parts[idx + 1]))
                    } else {
                        "Known Failures".to_string()
                    }
                },
                "tests" => {
                    // Build category from path components
                    let mut category_parts = Vec::new();
                    for i in (idx + 1)..(parts.len() - 1) {
                        category_parts.push(Self::capitalize_words(parts[i]));
                    }
                    
                    if category_parts.is_empty() {
                        "Uncategorized".to_string()
                    } else {
                        category_parts.join(" › ")
                    }
                },
                _ => "Uncategorized".to_string()
            }
        } else {
            "Uncategorized".to_string()
        }
    }
    
    fn capitalize_words(s: &str) -> String {
        s.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
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
    pub selected_item: usize,  // Index in the flattened list view
    pub test_scroll: usize,
    pub output_scroll: usize,
    pub source_scroll: usize,
    pub asm_scroll: usize,
    pub ir_scroll: usize,
    pub details_scroll: usize,
    pub category_scroll: usize,

    // Categories and filtering
    pub categories: BTreeMap<String, CategoryView>,
    pub selected_category: Option<String>,
    pub selected_category_index: usize,
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
    pub help_scroll: usize,
}

impl TuiApp {
    pub fn new(test_config: TestConfig, tools: ToolPaths, bank_size: usize, timeout_secs: u64) -> Self {
        let categories = CategoryView::from_tests(&test_config.tests, &test_config.known_failures);
        let mut app = Self {
            focused_pane: FocusedPane::TestList,
            mode: AppMode::Normal,
            selected_tab: 0,

            filtered_tests: test_config.tests.clone(),
            filtered_failures: test_config.known_failures.clone(),
            test_config,
            tools,

            selected_test: 0,
            selected_item: 0,
            test_scroll: 0,
            output_scroll: 0,
            source_scroll: 0,
            asm_scroll: 0,
            ir_scroll: 0,
            details_scroll: 0,
            category_scroll: 0,

            categories,
            selected_category: None,
            selected_category_index: 0,
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
            help_scroll: 0,
        };

        app.apply_filters();
        app
    }

    pub fn apply_filters(&mut self) {
        // Filter by category
        if let Some(ref category_name) = self.selected_category {
            if let Some(category) = self.categories.get(category_name) {
                self.filtered_tests = category.tests.clone();
                self.filtered_failures = Vec::new();
            } else if category_name == "Known Failures" {
                self.filtered_tests = Vec::new();
                self.filtered_failures = self.test_config.known_failures.clone();
            } else {
                self.filtered_tests = Vec::new();
                self.filtered_failures = Vec::new();
            }
        } else {
            // No category selected, show all
            self.filtered_tests = self.test_config.tests.clone();
            self.filtered_failures = self.test_config.known_failures.clone();
        }

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
        match self.get_selected_item_type() {
            SelectedItemType::Test(test) => {
                test.file.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            }
            _ => None
        }
    }

    pub fn get_selected_test_path(&self) -> Option<PathBuf> {
        match self.get_selected_item_type() {
            SelectedItemType::Test(test) => Some(test.file),
            _ => None
        }
    }

    pub fn get_selected_test_details(&self) -> Option<TestCase> {
        match self.get_selected_item_type() {
            SelectedItemType::Test(test) => Some(test),
            _ => None
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
            self.ensure_selection_visible();
        }
    }

    pub fn move_selection_down(&mut self) {
        let total_items = self.get_total_visible_items();
        if self.selected_item < total_items.saturating_sub(1) {
            self.selected_item += 1;
            self.ensure_selection_visible();
        }
    }
    
    pub fn get_total_visible_items(&self) -> usize {
        let mut count = 0;
        
        if let Some(ref selected_cat) = self.selected_category {
            // Single category view
            count += 1; // Category header
            if let Some(category) = self.categories.get(selected_cat) {
                if category.expanded {
                    count += category.tests.len();
                }
            }
        } else {
            // All categories view
            for (_name, category) in &self.categories {
                count += 1; // Category header
                if category.expanded {
                    count += category.tests.len();
                }
            }
        }
        
        count
    }
    
    pub fn toggle_current_category(&mut self) {
        let mut current_idx = 0;
        
        if let Some(ref selected_cat) = self.selected_category {
            // Single category view - toggle it if header is selected
            if self.selected_item == 0 {
                if let Some(category) = self.categories.get_mut(selected_cat) {
                    category.expanded = !category.expanded;
                }
            }
        } else {
            // All categories view - find which category header is selected
            for (name, category) in self.categories.iter_mut() {
                if current_idx == self.selected_item {
                    // This is the selected category header
                    category.expanded = !category.expanded;
                    return;
                }
                current_idx += 1;
                
                if category.expanded {
                    current_idx += category.tests.len();
                    if current_idx > self.selected_item {
                        // Selected item is a test, not a header
                        return;
                    }
                }
            }
        }
    }
    
    pub fn get_selected_item_type(&self) -> SelectedItemType {
        let mut current_idx = 0;
        
        if let Some(ref selected_cat) = self.selected_category {
            // Single category view
            if self.selected_item == 0 {
                return SelectedItemType::Category(selected_cat.clone());
            }
            if let Some(category) = self.categories.get(selected_cat) {
                if category.expanded && self.selected_item > 0 && self.selected_item <= category.tests.len() {
                    if let Some(test) = category.tests.get(self.selected_item - 1) {
                        return SelectedItemType::Test(test.clone());
                    }
                }
            }
        } else {
            // All categories view
            for (name, category) in &self.categories {
                if current_idx == self.selected_item {
                    return SelectedItemType::Category(name.clone());
                }
                current_idx += 1;
                
                if category.expanded {
                    for test in &category.tests {
                        if current_idx == self.selected_item {
                            return SelectedItemType::Test(test.clone());
                        }
                        current_idx += 1;
                    }
                }
            }
        }
        
        SelectedItemType::None
    }

    pub fn ensure_selection_visible(&mut self) {
        // Adjust scroll to keep selection visible
        const VISIBLE_ITEMS: usize = 20; // Approximate visible items in list
        
        if self.selected_item < self.test_scroll {
            self.test_scroll = self.selected_item;
        } else if self.selected_item >= self.test_scroll + VISIBLE_ITEMS {
            self.test_scroll = self.selected_item.saturating_sub(VISIBLE_ITEMS - 1);
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

    pub fn select_category(&mut self, category_name: String) {
        self.selected_category = Some(category_name);
        self.apply_filters();
        self.show_categories = false;
        self.mode = AppMode::Normal;
    }
    
    pub fn clear_category(&mut self) {
        self.selected_category = None;
        self.apply_filters();
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

    pub fn move_category_selection_up(&mut self) {
        let total_categories = self.categories.len() + 1; // +1 for "All Tests" option
        if self.selected_category_index > 0 {
            self.selected_category_index -= 1;
        } else {
            self.selected_category_index = total_categories - 1;
        }
    }

    pub fn move_category_selection_down(&mut self) {
        let total_categories = self.categories.len() + 1; // +1 for "All Tests" option
        self.selected_category_index = (self.selected_category_index + 1) % total_categories;
    }

    pub fn select_current_category(&mut self) {
        if self.selected_category_index == 0 {
            // "All Tests" option
            self.clear_category();
        } else {
            // Get the category at the current index
            let category_names: Vec<String> = self.categories.keys().cloned().collect();
            if let Some(category_name) = category_names.get(self.selected_category_index - 1) {
                self.select_category(category_name.clone());
            }
        }
    }
}