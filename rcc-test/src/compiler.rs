use crate::command::run_command_sync;
use crate::config::{Backend, RunConfig};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Paths to various tools
#[derive(Debug, Clone)]
pub struct ToolPaths {
    pub rcc: PathBuf,
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

    let asm_file = tools.build_dir.join(format!("{basename}.asm"));
    let ir_file = tools.build_dir.join(format!("{basename}.ir"));
    let pobj_file = tools.build_dir.join(format!("{basename}.pobj"));

    // Clean up previous files
    let _ = std::fs::remove_file(&asm_file);
    let _ = std::fs::remove_file(&ir_file);
    let _ = std::fs::remove_file(&pobj_file);

    // Step 1: Compile C to assembly
    let cmd = format!(
        "{} compile {} -o {} --save-ir --ir-output {}",
        tools.rcc.display(),
        c_file.display(),
        asm_file.display(),
        ir_file.display()
    );

    let result = run_command_sync(&cmd, config.timeout_secs)?;
    if result.exit_code != 0 {
        return Ok(CompilationResult {
            success: false,
            output: String::new(),
            has_provenance_warning: false,
            error_message: Some(format!("Compilation failed: {}", result.stderr)),
        });
    }

    // Check for provenance warnings
    let has_provenance_warning = if asm_file.exists() {
        let asm_content = std::fs::read_to_string(&asm_file)?;
        asm_content.contains("WARNING: Assuming unknown pointer points to global memory")
    } else {
        false
    };

    // Step 2: Assemble to object
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

    // Step 3: Link and run
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