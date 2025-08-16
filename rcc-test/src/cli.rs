use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "rct",
    about = "Test runner for the Ripple C compiler",
    long_about = "rct - Ripple C Compiler Test runner\n\nA fast, parallel test runner for the Ripple C compiler with support for both Brainfuck and RVM backends.",
    version,
    author
)]
pub struct Cli {
    /// Test names to run (without path or .c extension)
    #[arg(value_name = "TEST")]
    pub tests: Vec<String>,

    /// Execution backend
    #[arg(short, long, default_value = "rvm")]
    pub backend: BackendArg,

    /// Timeout in seconds for test execution
    #[arg(short, long, default_value = "2")]
    pub timeout: u64,

    /// Bank size for assembler
    #[arg(long, default_value = "16384")]
    pub bank_size: usize,

    /// Show output from test programs as they run
    #[arg(short, long)]
    pub verbose: bool,

    /// Don't clean up generated files after tests
    #[arg(long)]
    pub no_cleanup: bool,

    /// Disable parallel test execution
    #[arg(long)]
    pub no_parallel: bool,

    /// Use debug mode (RVM with -t flag)
    #[arg(short, long)]
    pub debug: bool,

    /// Path to tests.json file
    #[arg(long, default_value = "c-test/tests.json")]
    pub tests_file: PathBuf,

    /// Build directory for artifacts
    #[arg(long, default_value = "c-test/build")]
    pub build_dir: PathBuf,

    /// Project root directory
    #[arg(long)]
    pub project_root: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run tests (default if no command specified)
    Run {
        /// Run tests matching a pattern
        #[arg(short, long)]
        filter: Option<String>,
    },
    
    /// Add a new test to tests.json
    Add {
        /// Test file path
        file: PathBuf,
        
        /// Expected output (use \n for newlines)
        expected: Option<String>,
        
        /// Test doesn't use runtime
        #[arg(long)]
        no_runtime: bool,
        
        /// Test description
        #[arg(short, long)]
        description: Option<String>,
    },
    
    /// Clean build directory
    Clean,
    
    /// List all available tests
    List {
        /// Show only test names (no details)
        #[arg(short, long)]
        names_only: bool,
        
        /// Include known failures
        #[arg(short, long)]
        include_failures: bool,
    },
    
    /// Build and run a single test interactively
    Debug {
        /// Test name or file path
        test: String,
    },
    
    /// Build runtime library
    BuildRuntime,
    
    /// Show statistics about the test suite
    Stats,
    
    /// Rename a test (updates both JSON and file)
    Rename {
        /// Current test name or path
        old_name: String,
        
        /// New test name or path
        new_name: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BackendArg {
    Bf,
    Rvm,
}

impl BackendArg {
    pub fn to_backend(&self) -> crate::config::Backend {
        match self {
            BackendArg::Bf => crate::config::Backend::Brainfuck,
            BackendArg::Rvm => crate::config::Backend::Rvm,
        }
    }
}