use anyhow::{Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use which::which;

#[derive(Parser, Debug)]
#[command(name = "rbt")]
#[command(about = "Ripple Build Tool - A frontend for the Ripple assembly toolchain", long_about = None)]
struct Args {
    /// Assembly source files to build
    #[arg(required = true)]
    sources: Vec<PathBuf>,

    /// Output file (defaults to first source name with .bf extension)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Bank size for the assembler
    #[arg(short = 'b', long, default_value = "1024")]
    bank_size: u32,

    /// Output format (macro or binary)
    #[arg(short = 'f', long, default_value = "macro")]
    format: String,

    /// Run the resulting brainfuck program
    #[arg(short = 'r', long)]
    run: bool,

    /// Keep intermediate files for debugging
    #[arg(short = 'k', long)]
    keep_temp: bool,

    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Include debug information in output
    #[arg(short = 'd', long)]
    debug: bool,

    /// Generate standalone output (for macro format)
    #[arg(short = 's', long, default_value_t = true)]
    standalone: bool,
}

struct BuildContext {
    args: Args,
    temp_dir: Option<TempDir>,
    object_files: Vec<PathBuf>,
}

impl BuildContext {
    fn new(args: Args) -> Result<Self> {
        let temp_dir = if args.keep_temp {
            None
        } else {
            Some(TempDir::new()?)
        };

        Ok(Self {
            args,
            temp_dir,
            object_files: Vec::new(),
        })
    }

    fn get_temp_path(&self, filename: &str) -> PathBuf {
        if let Some(ref temp_dir) = self.temp_dir {
            temp_dir.path().join(filename)
        } else {
            PathBuf::from(filename)
        }
    }

    fn check_tool(name: &str) -> Result<PathBuf> {
        which(name).with_context(|| format!("Could not find '{}' in PATH", name))
    }

    fn run_command(&self, cmd: &mut Command) -> Result<()> {
        if self.args.verbose {
            eprintln!("Running: {:?}", cmd);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {}", stderr);
        }

        if self.args.verbose && !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(())
    }

    fn assemble(&mut self, source: &Path) -> Result<PathBuf> {
        let rasm = Self::check_tool("rasm")?;
        
        let stem = source
            .file_stem()
            .context("Invalid source filename")?
            .to_string_lossy();
        
        let output = self.get_temp_path(&format!("{}.pobj", stem));
        
        let mut cmd = Command::new(rasm);
        cmd.arg("assemble")
            .arg("-b")
            .arg(self.args.bank_size.to_string())
            .arg(source)
            .arg("-o")
            .arg(&output);

        self.run_command(&mut cmd)
            .with_context(|| format!("Failed to assemble {}", source.display()))?;

        Ok(output)
    }

    fn link(&self) -> Result<PathBuf> {
        let rlink = Self::check_tool("rlink")?;
        
        let output = if self.args.format == "macro" {
            self.get_temp_path("linked.bfm")
        } else {
            self.get_temp_path("linked.bin")
        };

        let mut cmd = Command::new(rlink);
        
        for obj in &self.object_files {
            cmd.arg(obj);
        }
        
        cmd.arg("--format").arg(&self.args.format);
        
        if self.args.standalone {
            cmd.arg("--standalone");
        }
        
        if self.args.debug {
            cmd.arg("--debug");
        }
        
        cmd.arg("-o").arg(&output);

        self.run_command(&mut cmd)
            .context("Failed to link object files")?;

        Ok(output)
    }

    fn expand(&self, macro_file: &Path) -> Result<PathBuf> {
        let bfm = Self::check_tool("bfm")?;
        
        let output = if let Some(ref user_output) = self.args.output {
            user_output.clone()
        } else {
            let first_source = &self.args.sources[0];
            let stem = first_source
                .file_stem()
                .context("Invalid source filename")?
                .to_string_lossy();
            PathBuf::from(format!("{}.bf", stem))
        };

        let mut cmd = Command::new(bfm);
        cmd.arg("expand")
            .arg(macro_file)
            .arg("-o")
            .arg(&output);

        self.run_command(&mut cmd)
            .context("Failed to expand macros")?;

        Ok(output)
    }

    fn expand_to_stdout(&self, macro_file: &Path) -> Result<String> {
        let bfm = Self::check_tool("bfm")?;
        
        let mut cmd = Command::new(bfm);
        cmd.arg("expand")
            .arg(macro_file);

        if self.args.verbose {
            eprintln!("Running: {:?}", cmd);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn run_bf_from_stdin(&self, bf_code: &str) -> Result<()> {
        let bf = Self::check_tool("bf")?;
        
        if self.args.verbose {
            eprintln!("Piping expanded code to bf interpreter");
        }

        use std::io::Write;
        use std::process::Stdio;
        
        let mut cmd = Command::new(bf)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        if let Some(mut stdin) = cmd.stdin.take() {
            stdin.write_all(bf_code.as_bytes())?;
            stdin.flush()?;
            drop(stdin); // Explicitly close stdin
        }

        let status = cmd.wait()?;
        
        if !status.success() {
            anyhow::bail!("Brainfuck execution failed");
        }

        Ok(())
    }

    fn build(&mut self) -> Result<Option<PathBuf>> {
        // Assemble all source files
        for source in &self.args.sources.clone() {
            if self.args.verbose {
                eprintln!("Assembling {}...", source.display());
            }
            let obj = self.assemble(source)?;
            self.object_files.push(obj);
        }

        // Link object files
        if self.args.verbose {
            eprintln!("Linking {} object files...", self.object_files.len());
        }
        let linked = self.link()?;

        // Expand macros if needed
        if self.args.format == "macro" {
            if self.args.verbose {
                eprintln!("Expanding macros...");
            }
            
            // If we're running the program, expand to stdout and pipe directly
            if self.args.run {
                let bf_code = self.expand_to_stdout(&linked)?;
                if self.args.verbose {
                    eprintln!("\nExecuting compiled program:");
                }
                self.run_bf_from_stdin(&bf_code)?;
                Ok(None) // No output file when running directly
            } else {
                // Otherwise, expand to file
                let output = self.expand(&linked)?;
                Ok(Some(output))
            }
        } else {
            // For binary format, just copy/move to final location
            let output = if let Some(ref user_output) = self.args.output {
                user_output.clone()
            } else {
                let first_source = &self.args.sources[0];
                let stem = first_source
                    .file_stem()
                    .context("Invalid source filename")?
                    .to_string_lossy();
                PathBuf::from(format!("{}.bin", stem))
            };
            
            std::fs::copy(&linked, &output)?;
            Ok(Some(output))
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Validate input files exist
    for source in &args.sources {
        if !source.exists() {
            anyhow::bail!("Source file not found: {}", source.display());
        }
    }

    // Validate format
    if args.format != "macro" && args.format != "binary" {
        anyhow::bail!("Invalid format: {}. Must be 'macro' or 'binary'", args.format);
    }

    // Check for conflicting options
    if args.run && args.format != "macro" {
        anyhow::bail!("Can only run macro format output (use -f macro)");
    }

    let mut ctx = BuildContext::new(args)?;
    
    let output = ctx.build()?;
    
    // Only print output path if we created a file
    if let Some(ref output_path) = output {
        if ctx.args.verbose || !ctx.args.run {
            println!("Output: {}", output_path.display());
        }
    }

    Ok(())
}