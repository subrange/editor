//! Ripple C99 Compiler Driver
//! 
//! Main entry point for the Ripple C99 compiler. For M1, this provides
//! a simple command-line interface for testing the backend code generation.

use clap::{Parser, Subcommand};
// use rcc_codegen::generate_assembly; // Not used currently
use rcc_backend::{lower_module_to_assembly_with_options, LoweringOptions};
use rcc_frontend::Frontend;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rcc")]
#[command(about = "Ripple C99 Compiler")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test the backend code generation with built-in examples
    Test {
        /// Which test to run
        #[arg(short, long, default_value = "hello")]
        test_name: String,
        
        /// Output file for generated assembly
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Generate assembly from simple IR (for M1 testing)
    GenerateAsm {
        /// Input IR description (JSON format planned)
        #[arg(short, long)]
        input: Option<PathBuf>,
        
        /// Output assembly file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Compile C99 source file
    Compile {
        /// Input C99 source file
        input: PathBuf,
        
        /// Output assembly file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Emit IR to stdout and exit (do not lower to assembly)
        #[arg(long)]
        emit_ir: bool,
        
        /// Print IR to stdout before lowering
        #[arg(long)]
        print_ir: bool,
        
        /// Save IR to file with .ir extension
        #[arg(long)]
        save_ir: bool,
        
        /// Specify output path for IR file (used with --save-ir)
        #[arg(long)]
        ir_output: Option<PathBuf>,
        
        /// Debug level (0=none, 1=basic, 2=verbose, 3=trace)
        #[arg(short, long, default_value = "0")]
        debug: u8,
        
        /// Bank size in cells (default: 4096)
        #[arg(long, default_value = "4096")]
        bank_size: u16,
        
        /// Enable spill/reload tracing for debugging register allocation
        #[arg(long)]
        trace_spills: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Test { test_name, output } => {
            if let Err(e) = run_test(&test_name, output.as_deref()) {
                eprintln!("Error running test: {e}");
                std::process::exit(1);
            }
        }
        Commands::GenerateAsm { input, output } => {
            if let Err(e) = generate_asm_command(input.as_deref(), output.as_deref()) {
                eprintln!("Error generating assembly: {e}");
                std::process::exit(1);
            }
        }
        Commands::Compile { input, output, emit_ir, print_ir, save_ir, ir_output, debug, bank_size, trace_spills } => {
            // Initialize logger based on debug level
            let log_level = match debug {
                0 => "error",
                1 => "warn",
                2 => "info",
                3 => "debug",
                _ => "trace",
            };
            
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
                .format_timestamp(None)
                .format_module_path(true)
                .format_target(false)
                .init();
            
            if let Err(e) = compile_c99_file(&input, output.as_deref(), emit_ir, print_ir, save_ir, ir_output.as_deref(), bank_size, trace_spills) {
                eprintln!("Error compiling C99 file: {e}");
                std::process::exit(1);
            }
        }
    }
}

fn run_test(_test_name: &str, _output_path: Option<&std::path::Path>) -> Result<(), Box<dyn std::error::Error>> {
    // Test command temporarily disabled while migrating to v2
    println!("Test command is temporarily disabled during v2 migration");
    println!("Please use the 'compile' command to test compilation");
    Ok(())
}

fn generate_asm_command(
    _input_path: Option<&std::path::Path>,
    _output_path: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement IR file parsing and assembly generation
    // For now, just show that the command exists
    println!("Generate assembly command - not yet implemented");
    println!("This will be implemented in a future milestone when we have a real IR format");
    Ok(())
}

fn compile_c99_file(
    input_path: &std::path::Path,
    output_path: Option<&std::path::Path>,
    emit_ir: bool,
    print_ir: bool,
    save_ir: bool,
    ir_output_path: Option<&std::path::Path>,
    bank_size: u16,
    trace_spills: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Don't print compilation message if we're just emitting IR
    if !emit_ir {
        println!("Compiling C99 file: {} (bank_size: {})", input_path.display(), bank_size);
    }
    
    // Read source file
    let source = fs::read_to_string(input_path)?;
    
    // Parse the source
    let ast = match Frontend::parse_source(&source) {
        Ok(ast) => {
            if !emit_ir {
                println!("Successfully parsed C99 source...");
                println!("Found {} top-level items", ast.items.len());
                
                for item in &ast.items {
                    match item {
                        rcc_frontend::TopLevelItem::Function(func) => {
                            println!("  Function: {} -> {}", func.name, func.return_type);
                        }
                        rcc_frontend::TopLevelItem::Declaration(decl) => {
                            println!("  Global variable: {} : {}", decl.name, decl.decl_type);
                        }
                        rcc_frontend::TopLevelItem::TypeDefinition { name, .. } => {
                            println!("  Type definition: {name}");
                        }
                    }
                }
            }
            ast
        }
        Err(e) => {
            eprintln!("Parse error: {e}");
            return Err(e.into());
        }
    };
    
    // Try to compile to IR - if it fails, return an error
    if !emit_ir {
        println!("\n💀 Attempting full compilation pipeline");
    }
    match Frontend::compile_to_ir(&source, input_path.file_stem().unwrap().to_str().unwrap()) {
        Ok(ir_module) => {
            if !emit_ir {
                println!("💫 Successfully generated IR");
                println!("🦄 Module contains {} functions", ir_module.functions.len());
            }
            
            // Format IR output
            let mut ir_output = String::new();
            
            // Add global declarations at the top for better debugging
            if !ir_module.globals.is_empty() {
                ir_output.push_str("; Global variables:\n");
                for global in &ir_module.globals {
                    ir_output.push_str(&format!("; @{} : {:?}", global.name, global.var_type));
                    if let Some(init) = &global.initializer {
                        // Show initializer in a readable format
                        match init {
                            rcc_frontend::ir::Value::Constant(val) => {
                                ir_output.push_str(&format!(" = {val}"));
                            }
                            rcc_frontend::ir::Value::ConstantArray(values) => {
                                // If it looks like a string, show it as a string
                                if values.last() == Some(&0) && 
                                   values[..values.len().saturating_sub(1)].iter()
                                       .all(|&v| (32..=126).contains(&v)) {
                                    let s: String = values[..values.len().saturating_sub(1)].iter()
                                        .map(|&c| c as u8 as char).collect();
                                    ir_output.push_str(&format!(" = \"{s}\""));
                                } else {
                                    ir_output.push_str(&format!(" = {values:?}"));
                                }
                            }
                            _ => {
                                ir_output.push_str(&format!(" = {init:?}"));
                            }
                        }
                    }
                    ir_output.push('\n');
                }
                ir_output.push('\n');
            }
            
            // String literals are included in globals now, so we don't need a separate section
            
            for func in &ir_module.functions {
                ir_output.push_str(&format!("define {} {{\n", func.name));
                for (param_id, param_type) in &func.parameters {
                    ir_output.push_str(&format!("  param %{param_id}: {param_type:?}\n"));
                }
                for block in &func.blocks {
                    ir_output.push_str(&format!("L{}:\n", block.id));
                    for inst in &block.instructions {
                        ir_output.push_str(&format!("  {inst}\n"));
                    }
                }
                ir_output.push_str("}\n");
            }
            
            // If --emit-ir is set, just print IR and exit
            if emit_ir {
                print!("{ir_output}");
                return Ok(());
            }
            
            // Print IR if requested (when not using --emit-ir)
            if print_ir {
                println!("\n=== IR Output ===");
                print!("{ir_output}");
                println!("=== End IR ===\n");
            }
            
            // Save IR to file if requested
            if save_ir {
                let ir_path = if let Some(path) = ir_output_path {
                    // Use the specified IR output path
                    path.to_path_buf()
                } else {
                    // Default: save next to input file with .ir extension
                    let mut path = input_path.to_path_buf();
                    path.set_extension("ir");
                    path
                };
                fs::write(&ir_path, &ir_output)?;
                println!("IR saved to: {}", ir_path.display());
            }
            
            // Check if main function exists
            let has_main = ir_module.functions.iter().any(|f| f.name == "main");
            
            // Lower Module to assembly with bank_size parameter
            let options = LoweringOptions {
                bank_size,
                use_v2: true,
                trace_spills,
            };
            match lower_module_to_assembly_with_options(&ir_module, options) {
                Ok(asm_instructions) => {
                    println!("💕 Successfully lowered to assembly");
                    
                    // Generate assembly text
                    let asm_text = rcc_codegen::emit::emit_complete_program(asm_instructions, has_main)?;
                    
                    // Write to output file
                    let final_output_path = if let Some(path) = output_path {
                        path.to_path_buf()
                    } else {
                        let mut path = input_path.to_path_buf();
                        path.set_extension("asm");
                        path
                    };
                    
                    fs::write(&final_output_path, asm_text)?;
                    println!("Assembly written to: {}", final_output_path.display());
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error: Failed to lower IR to assembly: {e}");
                    eprintln!("Note: Code generation is still under development for complex features");
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            eprintln!("Error: Failed to generate IR: {e}");
            eprintln!("Note: Code generation is still under development for complex features");
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hello_world_generation() {
        // Test that we can generate hello world without panicking
        let result = run_test("hello", None);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_arithmetic_generation() {
        // Test that we can generate arithmetic test without panicking
        let result = run_test("arithmetic", None);
        assert!(result.is_ok());
    }
}