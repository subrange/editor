use colored::*;
use similar::{ChangeTag, TextDiff};

/// Test result status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    Passed,
    PassedWithWarnings,
    Failed,
    Skipped,
    Timeout,
    CompilationError,
    KnownFailure,
    UnexpectedPass,
}

impl TestStatus {
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            TestStatus::Passed | TestStatus::PassedWithWarnings | TestStatus::KnownFailure
        )
    }

    pub fn symbol(&self) -> ColoredString {
        match self {
            TestStatus::Passed => "✓".green(),
            TestStatus::PassedWithWarnings => "✓".yellow(),
            TestStatus::Failed => "✗".red(),
            TestStatus::Skipped => "⊘".dimmed(),
            TestStatus::Timeout => "⏱".red(),
            TestStatus::CompilationError => "✗".red(),
            TestStatus::KnownFailure => "✓".yellow(),
            TestStatus::UnexpectedPass => "✗".red(),
        }
    }

    pub fn description(&self) -> ColoredString {
        match self {
            TestStatus::Passed => "PASSED".green(),
            TestStatus::PassedWithWarnings => "PASSED WITH WARNINGS".yellow(),
            TestStatus::Failed => "FAILED".red(),
            TestStatus::Skipped => "SKIPPED".dimmed(),
            TestStatus::Timeout => "TIMEOUT".red(),
            TestStatus::CompilationError => "COMPILATION ERROR".red(),
            TestStatus::KnownFailure => "EXPECTED FAIL".yellow(),
            TestStatus::UnexpectedPass => "UNEXPECTED PASS".red(),
        }
    }
}

/// Test result for reporting
#[derive(Debug)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub message: Option<String>,
    pub actual_output: Option<String>,
    pub expected_output: Option<String>,
    pub duration_ms: u64,
}

impl TestResult {
    /// Print a single test result
    pub fn print(&self, verbose: bool) {
        println!("{} {}: {}", 
            self.status.symbol(), 
            self.name, 
            self.status.description()
        );

        if let Some(msg) = &self.message {
            if verbose || !self.status.is_success() {
                println!("  {}", msg.dimmed());
            }
        }

        if verbose {
            if let Some(output) = &self.actual_output {
                if !output.is_empty() {
                    println!("  Output:\n{}", indent_lines(output, 4));
                }
            }
        }

        if !self.status.is_success() {
            if let (Some(expected), Some(actual)) = (&self.expected_output, &self.actual_output) {
                if expected != actual {
                    self.print_diff(expected, actual);
                }
            }
        }
    }

    /// Print a diff between expected and actual output
    fn print_diff(&self, expected: &str, actual: &str) {
        println!("\n  {}", "Output mismatch:".red().bold());
        
        // Show first difference
        let expected_lines: Vec<&str> = expected.lines().collect();
        let actual_lines: Vec<&str> = actual.lines().collect();
        
        for (i, (exp, act)) in expected_lines.iter().zip(actual_lines.iter()).enumerate() {
            if exp != act {
                println!("  First difference at line {}:", i + 1);
                println!("    Expected: {}", format!("{:?}", exp).green());
                println!("    Got:      {}", format!("{:?}", act).red());
                break;
            }
        }

        if expected_lines.len() != actual_lines.len() {
            println!(
                "  Different number of lines: expected {}, got {}",
                expected_lines.len(),
                actual_lines.len()
            );
        }

        // Show a nice diff
        println!("\n  {}", "Diff:".yellow());
        let diff = TextDiff::from_lines(expected, actual);
        
        for change in diff.iter_all_changes().take(20) {
            let sign = match change.tag() {
                ChangeTag::Delete => "-".red(),
                ChangeTag::Insert => "+".green(),
                ChangeTag::Equal => " ".dimmed(),
            };
            print!("  {}{}", sign, change);
        }

        // Show raw comparison for debugging
        println!("\n  {}", "Raw comparison:".dimmed());
        println!("    Expected: {:?}", expected);
        println!("    Got:      {:?}", actual);
    }
}

/// Test suite summary
#[derive(Debug, Default)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub skipped: usize,
    pub known_failures: usize,
    pub not_found: usize,
    pub total_duration_ms: u64,
}

impl TestSummary {
    /// Add a test result to the summary
    pub fn add(&mut self, result: &TestResult) {
        self.total += 1;
        self.total_duration_ms += result.duration_ms;
        
        match result.status {
            TestStatus::Passed => self.passed += 1,
            TestStatus::PassedWithWarnings => {
                self.passed += 1;
                self.warnings += 1;
            }
            TestStatus::Failed | TestStatus::Timeout | TestStatus::CompilationError => {
                self.failed += 1;
            }
            TestStatus::Skipped => self.skipped += 1,
            TestStatus::KnownFailure => self.known_failures += 1,
            TestStatus::UnexpectedPass => self.failed += 1,
        }
    }

    /// Print the test summary
    pub fn print(&self) {
        println!("\n{}", "=".repeat(60));
        println!("{:^60}", "Test Results");
        println!("{}", "=".repeat(60));
        
        println!("Total:          {}", self.total);
        println!("Passed:         {}", self.passed.to_string().green());
        
        if self.failed > 0 {
            println!("Failed:         {}", self.failed.to_string().red());
        }
        
        if self.warnings > 0 {
            println!("Warnings:       {}", self.warnings.to_string().yellow());
        }
        
        if self.skipped > 0 {
            println!("Skipped:        {}", self.skipped.to_string().dimmed());
        }
        
        if self.known_failures > 0 {
            println!("Known failures: {}", self.known_failures.to_string().yellow());
        }
        
        if self.not_found > 0 {
            println!("Not found:      {}", self.not_found.to_string().red().bold());
        }
        
        let duration_secs = self.total_duration_ms as f64 / 1000.0;
        println!("Duration:       {:.2}s", duration_secs);
        
        println!("{}", "=".repeat(60));
        
        if self.failed == 0 && self.not_found == 0 {
            if self.total == 0 {
                println!("\n{}", "No tests were run!".yellow().bold());
            } else {
                println!("\n{}", "All tests passed!".green().bold());
            }
        } else {
            let mut issues = Vec::new();
            if self.failed > 0 {
                issues.push(format!("{} failed", self.failed));
            }
            if self.not_found > 0 {
                issues.push(format!("{} not found", self.not_found));
            }
            println!("\n{}", format!("{} tests had issues ({})", 
                self.failed + self.not_found, 
                issues.join(", ")).red().bold());
        }
    }

    /// Return exit code based on results
    pub fn exit_code(&self) -> i32 {
        if self.failed == 0 && self.not_found == 0 { 0 } else { 1 }
    }
}

/// Progress reporter for parallel execution
pub struct ProgressReporter {
    progress_bar: indicatif::ProgressBar,
}

impl ProgressReporter {
    pub fn new(total: usize) -> Self {
        let progress_bar = indicatif::ProgressBar::new(total as u64);
        progress_bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        
        Self { progress_bar }
    }

    pub fn update(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
        self.progress_bar.inc(1);
    }

    pub fn finish(&self) {
        self.progress_bar.finish_and_clear();
    }
}

/// Helper to indent lines
fn indent_lines(text: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    text.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<_>>()
        .join("\n")
}