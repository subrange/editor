//! Ripple C99 Compiler Driver
//! 
//! Main entry point for the Ripple C99 compiler. For M1, this provides
//! a simple command-line interface for testing the backend code generation.

use clap::{Parser, Subcommand};
use rcc_codegen::generate_assembly;
use rcc_ir::{lower_to_assembly, lowering::test_helpers, Module};
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
        
        /// Print IR to stdout before lowering
        #[arg(long)]
        print_ir: bool,
        
        /// Save IR to file with .ir extension
        #[arg(long)]
        save_ir: bool,
        
        /// Specify output path for IR file (used with --save-ir)
        #[arg(long)]
        ir_output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Test { test_name, output } => {
            if let Err(e) = run_test(&test_name, output.as_deref()) {
                eprintln!("Error running test: {}", e);
                std::process::exit(1);
            }
        }
        Commands::GenerateAsm { input, output } => {
            if let Err(e) = generate_asm_command(input.as_deref(), output.as_deref()) {
                eprintln!("Error generating assembly: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Compile { input, output, print_ir, save_ir, ir_output } => {
            if let Err(e) = compile_c99_file(&input, output.as_deref(), print_ir, save_ir, ir_output.as_deref()) {
                eprintln!("Error compiling C99 file: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn run_test(test_name: &str, output_path: Option<&std::path::Path>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running test: {}", test_name);
    
    let program = match test_name {
        "hello" => test_helpers::create_hello_world_ir(),
        "arithmetic" => test_helpers::create_arithmetic_ir(),
        _ => {
            return Err(format!("Unknown test: {}", test_name).into());
        }
    };
    
    // Show the IR
    println!("\nGenerated IR:");
    println!("{}", program.display());
    
    // Lower to assembly
    let asm_instructions = lower_to_assembly(program)?;
    
    // Generate assembly text
    let asm_text = rcc_codegen::emit::emit_complete_program(asm_instructions, true)?;
    
    println!("\nGenerated Assembly:");
    println!("{}", asm_text);
    
    // Write to file if requested
    if let Some(path) = output_path {
        fs::write(path, &asm_text)?;
        println!("\nAssembly written to: {}", path.display());
    }
    
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
    print_ir: bool,
    save_ir: bool,
    ir_output_path: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Compiling C99 file: {}", input_path.display());
    
    // Read source file
    let source = fs::read_to_string(input_path)?;
    
    // Parse the source
    let ast = match Frontend::parse_source(&source) {
        Ok(ast) => {
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
                        println!("  Type definition: {}", name);
                    }
                }
            }
            ast
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            return Err(e.into());
        }
    };
    
    // Try to compile to IR - if it fails, return an error
    println!("\n💀 Attempting full compilation pipeline");
    match Frontend::compile_to_ir(&source, input_path.file_stem().unwrap().to_str().unwrap()) {
        Ok(ir_module) => {
            println!("💫 Successfully generated IR");
            println!("🦄 Module contains {} functions", ir_module.functions.len());
            
            // Format IR output
            let mut ir_output = String::new();
            for func in &ir_module.functions {
                ir_output.push_str(&format!("define {} {{\n", func.name));
                for (param_id, param_type) in &func.parameters {
                    ir_output.push_str(&format!("  param %{}: {:?}\n", param_id, param_type));
                }
                for block in &func.blocks {
                    ir_output.push_str(&format!("L{}:\n", block.id));
                    for inst in &block.instructions {
                        ir_output.push_str(&format!("  {}\n", inst));
                    }
                }
                ir_output.push_str("}\n");
            }
            
            // Print IR if requested
            if print_ir {
                println!("\n=== IR Output ===");
                print!("{}", ir_output);
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
            
            // Lower Module to assembly
            match rcc_ir::lower_module_to_assembly(ir_module) {
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
                    eprintln!("Error: Failed to lower IR to assembly: {}", e);
                    eprintln!("Note: Code generation is still under development for complex features");
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            eprintln!("Error: Failed to generate IR: {}", e);
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