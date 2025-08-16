use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use rcc_test::cli::{Cli, Command};
use rcc_test::command::run_command_sync;
use rcc_test::compiler::{build_runtime, ToolPaths};
use rcc_test::config::{self, RunConfig};
use rcc_test::runner::{cleanup_build_dir, TestRunner};
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", "Error".red().bold(), e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Determine project root
    let project_root = if let Some(root) = cli.project_root.clone() {
        root
    } else {
        detect_project_root()?
    };

    // Create tool paths
    let tools = ToolPaths::new(&project_root, &cli.build_dir);

    // Ensure build directory exists
    std::fs::create_dir_all(&cli.build_dir)?;

    // Handle subcommands
    match cli.command {
        Some(Command::Clean) => {
            println!("Cleaning build directory...");
            let count = cleanup_build_dir(&cli.build_dir)?;
            println!("Removed {} files", count);
            return Ok(());
        }

        Some(Command::BuildRuntime) => {
            println!("Building runtime library (bank_size: {})...", cli.bank_size);
            build_runtime(&tools, cli.bank_size)?;
            println!("{}", "Runtime built successfully".green());
            return Ok(());
        }

        Some(Command::Add { file, expected, no_runtime, description }) => {
            add_test(&cli.tests_file, file, expected, !no_runtime, description)?;
            return Ok(());
        }

        Some(Command::List { names_only, include_failures }) => {
            list_tests(&cli.tests_file, names_only, include_failures)?;
            return Ok(());
        }

        Some(Command::Stats) => {
            show_stats(&cli.tests_file)?;
            return Ok(());
        }
        
        Some(Command::Rename { ref old_name, ref new_name }) => {
            rename_test(&cli.tests_file, old_name, new_name)?;
            return Ok(());
        }

        Some(Command::Debug { ref test }) => {
            debug_test(&test, &cli, &tools)?;
            return Ok(());
        }

        Some(Command::Run { ref filter }) => {
            // Check if debug mode is requested with specific tests
            if cli.debug && !cli.tests.is_empty() {
                // Run each test in debug mode
                for test in &cli.tests {
                    debug_test(test, &cli, &tools)?;
                }
            } else {
                run_tests(&cli, &tools, filter.clone())?;
            }
        }
        None => {
            // Check if debug mode is requested with specific tests
            if cli.debug && !cli.tests.is_empty() {
                // Run each test in debug mode
                for test in &cli.tests {
                    debug_test(test, &cli, &tools)?;
                }
            } else {
                // Default to run command with no filter
                run_tests(&cli, &tools, None)?;
            }
        }
    }

    Ok(())
}

fn detect_project_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    
    // Check if we're in the project root (contains c-test directory)
    if current_dir.join("c-test").exists() {
        return Ok(current_dir);
    }
    
    // Check if we're in c-test directory
    if current_dir.file_name() == Some(std::ffi::OsStr::new("c-test")) {
        if let Some(parent) = current_dir.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    
    // Check if we're in rcc-test directory
    if current_dir.file_name() == Some(std::ffi::OsStr::new("rcc-test")) {
        if let Some(parent) = current_dir.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    
    // Try parent directory
    if let Some(parent) = current_dir.parent() {
        if parent.join("c-test").exists() {
            return Ok(parent.to_path_buf());
        }
    }
    
    anyhow::bail!("Could not detect project root. Please run from project root or use --project-root")
}

fn run_tests(cli: &Cli, tools: &ToolPaths, filter: Option<String>) -> Result<()> {
    // Build runtime first
    println!("Building runtime library (bank_size: {})...", cli.bank_size);
    build_runtime(tools, cli.bank_size)?;
    
    // Load test configuration
    let test_config = config::load_tests(&cli.tests_file)?;
    
    // Create run configuration
    let run_config = RunConfig {
        backend: cli.backend.to_backend(),
        timeout_secs: cli.timeout,
        bank_size: cli.bank_size,
        verbose: cli.verbose,
        no_cleanup: cli.no_cleanup,
        parallel: !cli.no_parallel,
        debug_mode: cli.debug,
    };
    
    // Create test runner
    let runner = TestRunner::new(run_config, tools.clone());
    
    // Run tests
    let summary = if !cli.tests.is_empty() {
        // Run specific tests
        runner.run_tests(&cli.tests, &test_config)?
    } else if let Some(pattern) = filter {
        // Filter tests by pattern
        let filtered_tests: Vec<String> = test_config
            .tests
            .iter()
            .filter(|t| {
                t.file.to_str()
                    .map(|s| s.contains(&pattern))
                    .unwrap_or(false)
            })
            .map(|t| {
                t.file.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string()
            })
            .collect();
        
        if filtered_tests.is_empty() {
            println!("No tests match filter: {}", pattern);
            return Ok(());
        }
        
        println!("Running {} tests matching '{}'", filtered_tests.len(), pattern);
        runner.run_tests(&filtered_tests, &test_config)?
    } else {
        // Run all tests
        runner.run_all(&test_config)?
    };
    
    // Print summary
    summary.print();
    
    // Clean up if requested
    if !cli.no_cleanup {
        let count = cleanup_build_dir(&cli.build_dir)?;
        if cli.verbose {
            println!("\nCleaned up {} files", count);
        }
    }
    
    process::exit(summary.exit_code())
}

fn add_test(
    tests_file: &Path,
    file: PathBuf,
    expected: Option<String>,
    use_runtime: bool,
    description: Option<String>,
) -> Result<()> {
    // Load existing config or create new
    let mut config = if tests_file.exists() {
        config::load_tests(tests_file)?
    } else {
        config::TestConfig {
            tests: Vec::new(),
            known_failures: Vec::new(),
        }
    };
    
    // Process escape sequences in expected output
    let expected = expected.map(|s| {
        s.replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r")
            .replace("\\\\", "\\")
    });
    
    // Normalize the file path and determine if it's a known failure
    let (file, is_known_failure) = if file.is_relative() {
        if file.starts_with("tests-known-failures") {
            (file, true)
        } else if file.starts_with("tests") {
            (file, false)
        } else {
            // Default to tests/ for regular tests
            (PathBuf::from("tests").join(file.file_name().unwrap()), false)
        }
    } else {
        // For absolute paths, check if it contains known-failures
        let is_failure = file.to_str()
            .map(|s| s.contains("known-failures"))
            .unwrap_or(false);
        (file, is_failure)
    };
    
    // Add to appropriate section
    let is_new = if is_known_failure || expected.is_none() {
        // Add to known failures
        if config.known_failures.iter().any(|f| f.file == file) {
            // Update existing
            if let Some(failure) = config.known_failures.iter_mut().find(|f| f.file == file) {
                if description.is_some() {
                    failure.description = description;
                }
                false
            } else {
                false
            }
        } else {
            config.known_failures.push(rcc_test::config::KnownFailure {
                file: file.clone(),
                description,
            });
            true
        }
    } else {
        // Add to regular tests
        config::add_test(
            &mut config,
            file.clone(),
            expected.clone(),
            use_runtime,
            description,
        )
    };
    
    // Save the config
    config::save_tests(&config, tests_file)?;
    
    if is_new {
        println!("Added new test: {}", file.display());
    } else {
        println!("Updated existing test: {}", file.display());
    }
    
    if let Some(exp) = expected {
        println!("Expected output: {:?}", exp);
    }
    
    Ok(())
}

fn list_tests(tests_file: &Path, names_only: bool, include_failures: bool) -> Result<()> {
    let config = config::load_tests(tests_file)?;
    
    if !names_only {
        println!("Available tests:");
        println!("{}", "-".repeat(60));
    }
    
    for test in &config.tests {
        if names_only {
            if let Some(stem) = test.file.file_stem() {
                println!("{}", stem.to_string_lossy());
            }
        } else {
            println!(
                "{} (runtime: {}, expected: {})",
                test.file.display(),
                if test.use_runtime { "yes" } else { "no" },
                if test.expected.is_some() { "defined" } else { "none" }
            );
            if let Some(desc) = &test.description {
                println!("  {}", desc.dimmed());
            }
        }
    }
    
    if include_failures {
        if !names_only {
            println!("\nKnown failures:");
            println!("{}", "-".repeat(60));
        }
        
        for failure in &config.known_failures {
            if names_only {
                if let Some(stem) = failure.file.file_stem() {
                    println!("{}", stem.to_string_lossy());
                }
            } else {
                println!("{}", failure.file.display());
                if let Some(desc) = &failure.description {
                    println!("  {}", desc.dimmed());
                }
            }
        }
    }
    
    Ok(())
}

fn show_stats(tests_file: &Path) -> Result<()> {
    let config = config::load_tests(tests_file)?;
    
    println!("Test Suite Statistics");
    println!("{}", "=".repeat(60));
    println!("Total tests:         {}", config.tests.len());
    
    let with_runtime = config.tests.iter().filter(|t| t.use_runtime).count();
    let without_runtime = config.tests.len() - with_runtime;
    
    println!("With runtime:        {}", with_runtime);
    println!("Without runtime:     {}", without_runtime);
    
    let with_expected = config.tests.iter().filter(|t| t.expected.is_some()).count();
    println!("With expected output: {}", with_expected);
    
    println!("Known failures:      {}", config.known_failures.len());
    
    // Calculate total lines of expected output
    let total_expected_lines: usize = config
        .tests
        .iter()
        .filter_map(|t| t.expected.as_ref())
        .map(|e| e.lines().count())
        .sum();
    
    println!("Total expected lines: {}", total_expected_lines);
    
    Ok(())
}

fn debug_test(test_name: &str, cli: &Cli, tools: &ToolPaths) -> Result<()> {
    // Build runtime first
    println!("Building runtime library...");
    build_runtime(tools, cli.bank_size)?;
    
    // Find the test file
    let test_path = find_test_file(test_name, &cli.tests_file)?;
    
    // Fix the path - prepend c-test if needed
    let actual_test_path = if test_path.is_relative() && !test_path.starts_with("c-test") {
        Path::new("c-test").join(&test_path)
    } else {
        test_path.clone()
    };
    
    println!("Compiling: {}", actual_test_path.display());
    
    // Compile the test
    let basename = test_path.file_stem().unwrap().to_str().unwrap();
    let asm_file = cli.build_dir.join(format!("{}.asm", basename));
    let ir_file = cli.build_dir.join(format!("{}.ir", basename));
    let pobj_file = cli.build_dir.join(format!("{}.pobj", basename));
    let bin_file = cli.build_dir.join(format!("{}.bin", basename));
    
    // Compile C to assembly - use actual_test_path instead of test_path
    let cmd = format!(
        "{} compile {} -o {} --save-ir --ir-output {}",
        tools.rcc.display(),
        actual_test_path.display(),
        asm_file.display(),
        ir_file.display()
    );
    
    println!("Running: {}", cmd.dimmed());
    let result = run_command_sync(&cmd, 30)?;
    if result.exit_code != 0 {
        anyhow::bail!("Compilation failed: {}", result.stderr);
    }
    
    // Assemble
    let cmd = format!(
        "{} assemble {} -o {} --bank-size {} --max-immediate 65535",
        tools.rasm.display(),
        asm_file.display(),
        pobj_file.display(),
        cli.bank_size
    );
    
    println!("Running: {}", cmd.dimmed());
    let result = run_command_sync(&cmd, 30)?;
    if result.exit_code != 0 {
        anyhow::bail!("Assembly failed: {}", result.stderr);
    }
    
    // Link
    let cmd = format!(
        "{} {} {} {} -f binary --bank-size {} -o {}",
        tools.rlink.display(),
        tools.crt0().display(),
        tools.libruntime().display(),
        pobj_file.display(),
        cli.bank_size,
        bin_file.display()
    );
    
    println!("Running: {}", cmd.dimmed());
    let result = run_command_sync(&cmd, 30)?;
    if result.exit_code != 0 {
        anyhow::bail!("Linking failed: {}", result.stderr);
    }
    
    println!("{}", format!("Successfully built {}", bin_file.display()).green());
    
    // Run with debugger
    println!("\nStarting debugger...");
    println!("{}", "-".repeat(60));
    
    let status = std::process::Command::new(&tools.rvm)
        .arg(&bin_file)
        .arg("-t")
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Debugger exited with error");
    }
    
    Ok(())
}

fn rename_test(tests_file: &Path, old_name: &str, new_name: &str) -> Result<()> {
    // Load test configuration
    let mut config = config::load_tests(tests_file)?;
    
    // Normalize names - strip .c extension if present
    let old_name = old_name.strip_suffix(".c").unwrap_or(old_name);
    let new_name = new_name.strip_suffix(".c").unwrap_or(new_name);
    
    // Find the test to rename
    let mut found_test = None;
    let mut found_index = None;
    
    for (idx, test) in config.tests.iter().enumerate() {
        if let Some(stem) = test.file.file_stem() {
            if stem.to_str() == Some(old_name) || test.file.to_str() == Some(old_name) {
                found_test = Some(test.clone());
                found_index = Some(idx);
                break;
            }
        }
    }
    
    // Check in known failures if not found in tests
    let mut is_known_failure = false;
    if found_test.is_none() {
        for (idx, failure) in config.known_failures.iter().enumerate() {
            if let Some(stem) = failure.file.file_stem() {
                if stem.to_str() == Some(old_name) || failure.file.to_str() == Some(old_name) {
                    // Convert to TestCase for uniform handling
                    found_test = Some(config::TestCase {
                        file: failure.file.clone(),
                        expected: None,
                        use_runtime: true,
                        description: failure.description.clone(),
                    });
                    found_index = Some(idx);
                    is_known_failure = true;
                    break;
                }
            }
        }
    }
    
    if let Some(test) = found_test {
        // Determine old and new paths
        let old_path = if test.file.is_relative() && !test.file.starts_with("c-test") {
            PathBuf::from("c-test").join(&test.file)
        } else {
            test.file.clone()
        };
        
        // Construct new path preserving directory structure
        let new_file_path = if test.file.starts_with("tests-known-failures") {
            PathBuf::from("tests-known-failures").join(format!("{}.c", new_name))
        } else if test.file.starts_with("tests") {
            PathBuf::from("tests").join(format!("{}.c", new_name))
        } else {
            PathBuf::from(format!("{}.c", new_name))
        };
        
        let new_path = if new_file_path.is_relative() && !new_file_path.starts_with("c-test") {
            PathBuf::from("c-test").join(&new_file_path)
        } else {
            new_file_path.clone()
        };
        
        // Check if old file exists
        if !old_path.exists() {
            anyhow::bail!("Source file {} does not exist", old_path.display());
        }
        
        // Check if new file already exists
        if new_path.exists() {
            anyhow::bail!("Target file {} already exists", new_path.display());
        }
        
        // Rename the actual file
        std::fs::rename(&old_path, &new_path)
            .context(format!("Failed to rename file from {} to {}", old_path.display(), new_path.display()))?;
        
        // Update the configuration
        if is_known_failure {
            if let Some(idx) = found_index {
                config.known_failures[idx].file = new_file_path;
            }
        } else {
            if let Some(idx) = found_index {
                config.tests[idx].file = new_file_path;
            }
        }
        
        // Save the updated configuration
        config::save_tests(&config, tests_file)?;
        
        println!("✓ Renamed test '{}' to '{}'", old_name, new_name);
        println!("  File: {} -> {}", old_path.display(), new_path.display());
        
        Ok(())
    } else {
        anyhow::bail!("Test '{}' not found in tests.json", old_name)
    }
}

fn find_test_file(test_name: &str, tests_file: &Path) -> Result<PathBuf> {
    // Try to load config and find test
    if let Ok(config) = config::load_tests(tests_file) {
        if let Some(test) = config::find_test(&config, test_name) {
            return Ok(test.file.clone());
        }
    }
    
    // Try direct paths
    let name = test_name.strip_suffix(".c").unwrap_or(test_name);
    let possible_paths = [
        format!("c-test/tests/{}.c", name),
        format!("c-test/tests-known-failures/{}.c", name),
        format!("tests/{}.c", name),
        format!("tests-known-failures/{}.c", name),
        test_name.to_string(),
    ];
    
    for path_str in &possible_paths {
        let path = Path::new(path_str);
        if path.exists() {
            return Ok(path.to_path_buf());
        }
    }
    
    anyhow::bail!("Test '{}' not found", test_name)
}
