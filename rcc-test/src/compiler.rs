use crate::command::run_command_sync;
use crate::config::{Backend, RunConfig};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Clean up compiler error messages for TUI display
fn clean_compiler_error(stderr: &str) -> String {
    // If it's a compiler panic, extract just the important message
    if stderr.contains("COMPILER BUG:") {
        // Extract the COMPILER BUG message line
        for line in stderr.lines() {
            if line.contains("COMPILER BUG:") {
                // Remove ANSI escape codes and clean up
                let cleaned = strip_ansi_escapes(line);
                return cleaned;
            }
        }
    }
    
    // For other errors, take the first few meaningful lines
    let mut result = Vec::new();
    let mut in_backtrace = false;
    
    for line in stderr.lines() {
        // Skip backtrace lines
        if line.contains("stack backtrace:") || line.contains("note: run with") {
            in_backtrace = true;
            continue;
        }
        if in_backtrace {
            continue;
        }
        
        // Skip thread panic location details
        if line.starts_with("thread '") && line.contains("panicked at") {
            // Extract just the panic message
            if let Some(msg_start) = line.find("panicked at ") {
                let msg = &line[msg_start + 12..];
                if let Some(comma_pos) = msg.find(", ") {
                    result.push(msg[..comma_pos].to_string());
                } else {
                    result.push(msg.to_string());
                }
            }
            continue;
        }
        
        // Add meaningful error lines (limit to prevent overflow)
        if !line.trim().is_empty() && result.len() < 3 {
            result.push(strip_ansi_escapes(line));
        }
    }
    
    if result.is_empty() {
        // Fallback: just take first line or a truncated version
        stderr.lines().next().map(strip_ansi_escapes).unwrap_or_else(|| "Unknown error".to_string())
    } else {
        result.join(" ")
    }
}

/// Strip ANSI escape codes from a string
fn strip_ansi_escapes(s: &str) -> String {
    // Remove ANSI escape sequences (colors, formatting, etc.)
    let mut result = String::new();
    let mut in_escape = false;
    
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if ch.is_alphabetic() {
                in_escape = false;
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Paths to various tools
#[derive(Debug, Clone)]
pub struct ToolPaths {
    pub rcc: PathBuf,
    pub rcpp: PathBuf,
    pub rasm: PathBuf,
    pub rlink: PathBuf,
    pub rvm: PathBuf,
    pub runtime_dir: PathBuf,
    pub build_dir: PathBuf,
}

impl ToolPaths {
    /// Create tool paths relative to project root
    pub fn new(project_root: &Path, build_dir: &Path) -> Self {
        Self {
            rcc: project_root.join("target/release/rcc"),
            rcpp: project_root.join("target/release/rcpp"),
            rasm: project_root.join("src/ripple-asm/target/release/rasm"),
            rlink: project_root.join("src/ripple-asm/target/release/rlink"),
            rvm: project_root.join("target/release/rvm"),
            runtime_dir: project_root.join("runtime"),
            build_dir: build_dir.to_path_buf(),
        }
    }

    pub fn crt0(&self) -> PathBuf {
        self.runtime_dir.join("crt0.pobj")
    }

    pub fn libruntime(&self) -> PathBuf {
        self.runtime_dir.join("libruntime.par")
    }
}

/// Build the runtime library
pub fn build_runtime(tools: &ToolPaths, bank_size: usize) -> Result<()> {
    let cmd = format!(
        "cd {} && make clean && make all BANK_SIZE={}",
        tools.runtime_dir.display(),
        bank_size
    );
    
    let result = run_command_sync(&cmd, 30)?;
    if result.exit_code != 0 {
        anyhow::bail!("Failed to build runtime: {}", result.stderr);
    }

    // Also build crt0.pobj
    let cmd = format!(
        "cd {} && make crt0.pobj BANK_SIZE={}",
        tools.runtime_dir.display(),
        bank_size
    );
    
    let result = run_command_sync(&cmd, 30)?;
    if result.exit_code != 0 {
        anyhow::bail!("Failed to build crt0.pobj: {}", result.stderr);
    }

    Ok(())
}

/// Compilation result
#[derive(Debug)]
pub struct CompilationResult {
    pub success: bool,
    pub output: String,
    pub has_provenance_warning: bool,
    pub error_message: Option<String>,
}

/// Compile a C file to binary without running it
/// This is used by debug_test and exec_program commands
pub fn compile_c_to_binary(
    c_file: &Path,
    tools: &ToolPaths,
    bank_size: usize,
    use_runtime: bool,
    timeout_secs: u64,
) -> Result<PathBuf> {
    let basename = c_file
        .file_stem()
        .context("Invalid C file path")?
        .to_str()
        .context("Non-UTF8 filename")?;

    let preprocessed_file = tools.build_dir.join(format!("{basename}.pp.c"));
    let asm_file = tools.build_dir.join(format!("{basename}.asm"));
    let ir_file = tools.build_dir.join(format!("{basename}.ir"));
    let pobj_file = tools.build_dir.join(format!("{basename}.pobj"));
    let bin_file = tools.build_dir.join(format!("{basename}.bin"));

    // Step 1: Preprocess the C file (with runtime include directory)
    let cmd = format!(
        "{} {} -o {} -I {}",
        tools.rcpp.display(),
        c_file.display(),
        preprocessed_file.display(),
        tools.runtime_dir.join("include").display()
    );

    let result = run_command_sync(&cmd, timeout_secs)?;
    if result.exit_code != 0 {
        anyhow::bail!("Preprocessing failed: {}", result.stderr);
    }

    // Step 2: Compile preprocessed C to assembly (with --no-preprocess since we already preprocessed)
    let cmd = format!(
        "{} compile {} -o {} --save-ir --ir-output {} --no-preprocess",
        tools.rcc.display(),
        preprocessed_file.display(),
        asm_file.display(),
        ir_file.display()
    );

    let result = run_command_sync(&cmd, timeout_secs)?;
    if result.exit_code != 0 {
        anyhow::bail!("Compilation failed: {}", result.stderr);
    }

    // Step 3: Assemble to object
    let cmd = format!(
        "{} assemble {} -o {} --bank-size {} --max-immediate 65535",
        tools.rasm.display(),
        asm_file.display(),
        pobj_file.display(),
        bank_size
    );

    let result = run_command_sync(&cmd, timeout_secs)?;
    if result.exit_code != 0 {
        anyhow::bail!("Assembly failed: {}", result.stderr);
    }

    // Step 4: Link to binary
    let link_cmd = if use_runtime {
        format!(
            "{} {} {} {} -f binary --bank-size {} -o {}",
            tools.rlink.display(),
            tools.crt0().display(),
            tools.libruntime().display(),
            pobj_file.display(),
            bank_size,
            bin_file.display()
        )
    } else {
        format!(
            "{} {} {} -f binary --bank-size {} -o {}",
            tools.rlink.display(),
            tools.crt0().display(),
            pobj_file.display(),
            bank_size,
            bin_file.display()
        )
    };

    let result = run_command_sync(&link_cmd, timeout_secs)?;
    if result.exit_code != 0 {
        anyhow::bail!("Linking failed: {}", result.stderr);
    }

    Ok(bin_file)
}

/// Compile a C file to executable
pub fn compile_c_file(
    c_file: &Path,
    tools: &ToolPaths,
    config: &RunConfig,
    use_runtime: bool,
) -> Result<CompilationResult> {
    let basename = c_file
        .file_stem()
        .context("Invalid C file path")?
        .to_str()
        .context("Non-UTF8 filename")?;

    let preprocessed_file = tools.build_dir.join(format!("{basename}.pp.c"));
    let asm_file = tools.build_dir.join(format!("{basename}.asm"));
    let ir_file = tools.build_dir.join(format!("{basename}.ir"));
    let pobj_file = tools.build_dir.join(format!("{basename}.pobj"));

    // Clean up previous files
    let _ = std::fs::remove_file(&preprocessed_file);
    let _ = std::fs::remove_file(&asm_file);
    let _ = std::fs::remove_file(&ir_file);
    let _ = std::fs::remove_file(&pobj_file);

    // Step 1: Preprocess the C file (with runtime include directory)
    let cmd = format!(
        "{} {} -o {} -I {}",
        tools.rcpp.display(),
        c_file.display(),
        preprocessed_file.display(),
        tools.runtime_dir.join("include").display()
    );

    let result = run_command_sync(&cmd, config.timeout_secs)?;
    if result.exit_code != 0 {
        return Ok(CompilationResult {
            success: false,
            output: String::new(),
            has_provenance_warning: false,
            error_message: Some(format!("Preprocessing failed: {}", result.stderr)),
        });
    }

    // Step 2: Compile preprocessed C to assembly (with --no-preprocess since we already preprocessed)
    let cmd = format!(
        "{} compile {} -o {} --save-ir --ir-output {} --no-preprocess",
        tools.rcc.display(),
        preprocessed_file.display(),
        asm_file.display(),
        ir_file.display()
    );

    let result = run_command_sync(&cmd, config.timeout_secs)?;
    if result.exit_code != 0 {
        // Clean up the error message to be TUI-friendly
        let error_msg = clean_compiler_error(&result.stderr);
        return Ok(CompilationResult {
            success: false,
            output: String::new(),
            has_provenance_warning: false,
            error_message: Some(format!("Compilation failed: {}", error_msg)),
        });
    }

    // Check for provenance warnings
    let has_provenance_warning = if asm_file.exists() {
        let asm_content = std::fs::read_to_string(&asm_file)?;
        asm_content.contains("WARNING: Assuming unknown pointer points to global memory")
    } else {
        false
    };

    // Step 3: Assemble to object
    let cmd = format!(
        "{} assemble {} -o {} --bank-size {} --max-immediate 65535",
        tools.rasm.display(),
        asm_file.display(),
        pobj_file.display(),
        config.bank_size
    );

    let result = run_command_sync(&cmd, config.timeout_secs)?;
    if result.exit_code != 0 {
        return Ok(CompilationResult {
            success: false,
            output: String::new(),
            has_provenance_warning,
            error_message: Some(format!("Assembly failed: {}", result.stderr)),
        });
    }

    // Step 4: Link and run
    match config.backend {
        Backend::Rvm => {
            compile_and_run_rvm(
                basename,
                &pobj_file,
                tools,
                config,
                use_runtime,
                has_provenance_warning,
            )
        }
        Backend::Brainfuck => {
            compile_and_run_bf(
                basename,
                &pobj_file,
                tools,
                config,
                use_runtime,
                has_provenance_warning,
            )
        }
    }
}

fn compile_and_run_rvm(
    basename: &str,
    pobj_file: &Path,
    tools: &ToolPaths,
    config: &RunConfig,
    use_runtime: bool,
    has_provenance_warning: bool,
) -> Result<CompilationResult> {
    let bin_file = tools.build_dir.join(format!("{basename}.bin"));

    // Link to binary
    let link_cmd = if use_runtime {
        format!(
            "{} {} {} {} -f binary --bank-size {} -o {}",
            tools.rlink.display(),
            tools.crt0().display(),
            tools.libruntime().display(),
            pobj_file.display(),
            config.bank_size,
            bin_file.display()
        )
    } else {
        format!(
            "{} {} {} -f binary --bank-size {} -o {}",
            tools.rlink.display(),
            tools.crt0().display(),
            pobj_file.display(),
            config.bank_size,
            bin_file.display()
        )
    };

    let result = run_command_sync(&link_cmd, config.timeout_secs)?;
    if result.exit_code != 0 {
        return Ok(CompilationResult {
            success: false,
            output: String::new(),
            has_provenance_warning,
            error_message: Some(format!("Linking failed: {}", result.stderr)),
        });
    }

    // Run on RVM
    let rvm_flags = if config.debug_mode { "-t" } else { "" };
    let run_cmd = format!(
        "{} {} --memory 4294967296 {}",
        tools.rvm.display(),
        bin_file.display(),
        rvm_flags
    );

    let result = run_command_sync(&run_cmd, config.timeout_secs)?;
    
    // Save disassembly for debugging
    let _ = run_command_sync(
        &format!(
            "{} disassemble {} -o {}",
            tools.rasm.display(),
            bin_file.display(),
            tools.build_dir.join(format!("{basename}.disassembly.asm")).display()
        ),
        10,
    );

    if result.timed_out {
        Ok(CompilationResult {
            success: false,
            output: result.stdout,
            has_provenance_warning,
            error_message: Some(result.stderr),
        })
    } else {
        Ok(CompilationResult {
            success: true,
            output: result.stdout,
            has_provenance_warning,
            error_message: None,
        })
    }
}

fn compile_and_run_bf(
    basename: &str,
    pobj_file: &Path,
    tools: &ToolPaths,
    config: &RunConfig,
    use_runtime: bool,
    has_provenance_warning: bool,
) -> Result<CompilationResult> {
    let bf_file = tools.build_dir.join(format!("{basename}.bfm"));
    let expanded_file = tools.build_dir.join(format!("{basename}_expanded.bf"));

    // Link to macro format
    let link_cmd = if use_runtime {
        format!(
            "{} {} {} {} -f macro --standalone --bank-size {} -o {}",
            tools.rlink.display(),
            tools.crt0().display(),
            tools.libruntime().display(),
            pobj_file.display(),
            config.bank_size,
            bf_file.display()
        )
    } else {
        format!(
            "{} {} {} -f macro --standalone --bank-size {} -o {}",
            tools.rlink.display(),
            tools.crt0().display(),
            pobj_file.display(),
            config.bank_size,
            bf_file.display()
        )
    };

    let result = run_command_sync(&link_cmd, config.timeout_secs)?;
    if result.exit_code != 0 {
        return Ok(CompilationResult {
            success: false,
            output: String::new(),
            has_provenance_warning,
            error_message: Some(format!("Linking failed: {}", result.stderr)),
        });
    }

    // Expand macros
    let expand_cmd = format!(
        "bfm expand {} -o {}",
        bf_file.display(),
        expanded_file.display()
    );

    let result = run_command_sync(&expand_cmd, config.timeout_secs)?;
    if result.exit_code != 0 {
        return Ok(CompilationResult {
            success: false,
            output: String::new(),
            has_provenance_warning,
            error_message: Some(format!("Macro expansion failed: {}", result.stderr)),
        });
    }

    // Run brainfuck
    let run_cmd = format!(
        "bf {} --cell-size 16 --tape-size 150000000",
        expanded_file.display()
    );

    let result = run_command_sync(&run_cmd, config.timeout_secs)?;
    
    if result.timed_out {
        Ok(CompilationResult {
            success: false,
            output: result.stdout,
            has_provenance_warning,
            error_message: Some(result.stderr),
        })
    } else {
        Ok(CompilationResult {
            success: true,
            output: result.stdout,
            has_provenance_warning,
            error_message: None,
        })
    }
}