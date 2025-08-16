use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Test metadata stored in .meta.json files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetadata {
    pub expected: Option<String>,
    #[serde(default = "default_true")]
    pub use_runtime: bool,
    pub description: Option<String>,
    #[serde(default)]
    pub known_failure: bool,
    #[serde(default)]
    pub category: Option<String>,
}

/// Test configuration discovered from .meta.json files
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

/// Load test configuration by discovering .meta.json files
pub fn load_tests(_path: &Path) -> Result<TestConfig> {
    discover_tests()
}

/// Discover tests by scanning for .meta.json files
pub fn discover_tests() -> Result<TestConfig> {
    let mut tests = Vec::new();
    let mut known_failures = Vec::new();
    
    // Define the test directories to scan
    let test_dirs = vec![
        "c-test/tests",
        "c-test/tests-runtime", 
        "c-test/tests-known-failures",
        "c-test/known-failures",
        "c-test/examples",
    ];
    
    for dir in test_dirs {
        let dir_path = Path::new(dir);
        if dir_path.exists() {
            scan_directory_for_tests(dir_path, &mut tests, &mut known_failures)?;
        }
    }
    
    // Sort tests by path for consistent ordering
    tests.sort_by(|a, b| a.file.cmp(&b.file));
    known_failures.sort_by(|a, b| a.file.cmp(&b.file));
    
    Ok(TestConfig {
        tests,
        known_failures,
    })
}

/// Discover orphan tests (C files without .meta.json)
pub fn discover_orphan_tests() -> Result<Vec<TestCase>> {
    let mut orphans = Vec::new();
    
    // Define the test directories to scan
    let test_dirs = vec![
        "c-test/tests",
        "c-test/tests-runtime", 
        "c-test/tests-known-failures",
        "c-test/known-failures",
        "c-test/examples",
    ];
    
    for dir in test_dirs {
        let dir_path = Path::new(dir);
        if dir_path.exists() {
            scan_directory_for_orphans(dir_path, &mut orphans)?;
        }
    }
    
    // Sort orphans by path for consistent ordering
    orphans.sort_by(|a, b| a.file.cmp(&b.file));
    
    Ok(orphans)
}

/// Recursively scan a directory for test files with .meta.json
fn scan_directory_for_tests(
    dir: &Path,
    tests: &mut Vec<TestCase>,
    known_failures: &mut Vec<KnownFailure>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Skip build directory
            if path.file_name() == Some(std::ffi::OsStr::new("build")) {
                continue;
            }
            // Recursively scan subdirectories
            scan_directory_for_tests(&path, tests, known_failures)?;
        } else if path.extension() == Some(std::ffi::OsStr::new("c")) {
            // Check for corresponding .meta.json file
            let meta_path = path.with_extension("meta.json");
            if meta_path.exists() {
                // Load metadata
                let meta_content = std::fs::read_to_string(&meta_path)?;
                let metadata: TestMetadata = serde_json::from_str(&meta_content)?;
                
                // Get relative path from c-test directory
                let relative_path = if path.starts_with("c-test/") {
                    path.strip_prefix("c-test/")?.to_path_buf()
                } else if let Ok(rel) = path.strip_prefix(std::env::current_dir()?.join("c-test")) {
                    rel.to_path_buf()
                } else {
                    // Try to make it relative to c-test
                    let path_str = path.to_string_lossy();
                    if let Some(idx) = path_str.find("c-test/") {
                        PathBuf::from(&path_str[idx + 7..])
                    } else {
                        path.clone()
                    }
                };
                
                if metadata.known_failure {
                    known_failures.push(KnownFailure {
                        file: relative_path,
                        description: metadata.description,
                    });
                } else {
                    tests.push(TestCase {
                        file: relative_path,
                        expected: metadata.expected,
                        use_runtime: metadata.use_runtime,
                        description: metadata.description,
                    });
                }
            }
        }
    }
    
    Ok(())
}

/// Recursively scan a directory for orphan test files (C files without .meta.json)
fn scan_directory_for_orphans(
    dir: &Path,
    orphans: &mut Vec<TestCase>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Skip build directory
            if path.file_name() == Some(std::ffi::OsStr::new("build")) {
                continue;
            }
            // Recursively scan subdirectories
            scan_directory_for_orphans(&path, orphans)?;
        } else if path.extension() == Some(std::ffi::OsStr::new("c")) {
            // Check for corresponding .meta.json file
            let meta_path = path.with_extension("meta.json");
            if !meta_path.exists() {
                // This is an orphan test - create a TestCase with minimal info
                let relative_path = if path.starts_with("c-test/") {
                    path.strip_prefix("c-test/")?.to_path_buf()
                } else if let Ok(rel) = path.strip_prefix(std::env::current_dir()?.join("c-test")) {
                    rel.to_path_buf()
                } else {
                    // Try to make it relative to c-test
                    let path_str = path.to_string_lossy();
                    if let Some(idx) = path_str.find("c-test/") {
                        PathBuf::from(&path_str[idx + 7..])
                    } else {
                        path.clone()
                    }
                };
                
                orphans.push(TestCase {
                    file: relative_path,
                    expected: None,
                    use_runtime: true, // Default to using runtime
                    description: Some("[ORPHAN] Test without metadata".to_string()),
                });
            }
        }
    }
    
    Ok(())
}

/// Save test configuration to individual .meta.json files
pub fn save_tests(config: &TestConfig, _path: &Path) -> Result<()> {
    // Save regular tests
    for test in &config.tests {
        save_test_metadata(&test.file, &test.expected, test.use_runtime, &test.description, false)?;
    }
    
    // Save known failures
    for failure in &config.known_failures {
        save_test_metadata(&failure.file, &None, true, &failure.description, true)?;
    }
    
    Ok(())
}

/// Save metadata for a single test
fn save_test_metadata(
    file: &Path,
    expected: &Option<String>,
    use_runtime: bool,
    description: &Option<String>,
    known_failure: bool,
) -> Result<()> {
    // Construct the full path
    let full_path = if file.is_relative() && !file.starts_with("c-test") {
        Path::new("c-test").join(file)
    } else {
        file.to_path_buf()
    };
    
    let meta_path = full_path.with_extension("meta.json");
    
    // Create metadata
    let mut metadata = HashMap::new();
    
    if known_failure {
        metadata.insert("known_failure", serde_json::Value::Bool(true));
    } else {
        if let Some(exp) = expected {
            metadata.insert("expected", serde_json::Value::String(exp.clone()));
        }
        metadata.insert("use_runtime", serde_json::Value::Bool(use_runtime));
    }
    
    if let Some(desc) = description {
        metadata.insert("description", serde_json::Value::String(desc.clone()));
    }
    
    // Write the metadata file
    let content = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(meta_path, content)?;
    
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