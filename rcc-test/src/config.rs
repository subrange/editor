use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Test configuration loaded from tests.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub tests: Vec<TestCase>,
    pub known_failures: Vec<KnownFailure>,
}

/// A single test case definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub file: PathBuf,
    pub expected: Option<String>,
    #[serde(default = "default_true")]
    pub use_runtime: bool,
    pub description: Option<String>,
}

/// A known failure test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownFailure {
    pub file: PathBuf,
    pub description: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Configuration for running tests
#[derive(Debug, Clone)]
pub struct RunConfig {
    pub backend: Backend,
    pub timeout_secs: u64,
    pub bank_size: usize,
    pub verbose: bool,
    pub no_cleanup: bool,
    pub parallel: bool,
    pub debug_mode: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            backend: Backend::Rvm,
            timeout_secs: 2,
            bank_size: 16384,
            verbose: false,
            no_cleanup: false,
            parallel: true,
            debug_mode: false,
        }
    }
}

/// Execution backend for tests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Brainfuck,
    Rvm,
}

impl Backend {
    pub fn as_str(&self) -> &str {
        match self {
            Backend::Brainfuck => "bf",
            Backend::Rvm => "rvm",
        }
    }
}

/// Load test configuration from JSON file
pub fn load_tests(path: &Path) -> Result<TestConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: TestConfig = serde_json::from_str(&content)?;
    Ok(config)
}

/// Save test configuration to JSON file
pub fn save_tests(config: &TestConfig, path: &Path) -> Result<()> {
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Find test by name (without path or extension)
pub fn find_test<'a>(
    config: &'a TestConfig,
    name: &str,
) -> Option<&'a TestCase> {
    let name = name.strip_suffix(".c").unwrap_or(name);
    
    config.tests.iter().find(|test| {
        let file_stem = test.file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        file_stem == name
    })
}

/// Add a new test to the configuration
pub fn add_test(
    config: &mut TestConfig,
    file: PathBuf,
    expected: Option<String>,
    use_runtime: bool,
    description: Option<String>,
) -> bool {
    // Check if test already exists
    if config.tests.iter().any(|t| t.file == file) {
        // Update existing test
        if let Some(test) = config.tests.iter_mut().find(|t| t.file == file) {
            test.expected = expected;
            test.use_runtime = use_runtime;
            if description.is_some() {
                test.description = description;
            }
            return false; // Updated existing
        }
    }
    
    config.tests.push(TestCase {
        file,
        expected,
        use_runtime,
        description,
    });
    true // Added new
}