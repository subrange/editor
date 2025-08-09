use clap::{Parser, Subcommand};
use ripple_asm::{RippleAssembler, AssemblerOptions, MacroFormatter};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rasm")]
#[command(about = "Ripple Assembler - Assembles Ripple assembly files")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Assemble source file to object file
    Assemble {
        /// Input assembly file (.asm)
        input: PathBuf,
        
        /// Output object file (.pobj)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Output format (json, binary, macro)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// Bank size
        #[arg(short, long, default_value = "16")]
        bank_size: u16,
        
        /// Maximum immediate value
        #[arg(short, long, default_value = "65535")]
        max_immediate: u32,
        
        /// Data section offset
        #[arg(short, long, default_value = "2")]
        data_offset: u16,
        
        /// Case insensitive parsing
        #[arg(long, default_value = "true")]
        case_insensitive: bool,
    },
    
    /// Convert object file to macro format
    Format {
        /// Input object file (.pobj)
        input: PathBuf,
        
        /// Output macro file (.bfm)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Validate assembly file
    Check {
        /// Input assembly file
        input: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Assemble { 
            input, 
            output, 
            format, 
            bank_size, 
            max_immediate,
            data_offset,
            case_insensitive,
        } => {
            let source = fs::read_to_string(&input)?;
            
            let options = AssemblerOptions {
                case_insensitive,
                start_bank: 0,
                bank_size,
                max_immediate,
                data_offset,
            };
            
            let assembler = RippleAssembler::new(options);
            
            match assembler.assemble(&source) {
                Ok(obj) => {
                    let output_path = output.unwrap_or_else(|| {
                        let mut path = input.clone();
                        path.set_extension(match format.as_str() {
                            "binary" => "bin",
                            "macro" => "bfm",
                            _ => "pobj",
                        });
                        path
                    });
                    
                    match format.as_str() {
                        "json" => {
                            let json = serde_json::to_string_pretty(&obj)?;
                            fs::write(&output_path, json)?;
                            println!("✓ Assembled to {}", output_path.display());
                        }
                        "binary" => {
                            match assembler.assemble_to_binary(&source) {
                                Ok(binary) => {
                                    fs::write(&output_path, binary)?;
                                    println!("✓ Assembled to binary {}", output_path.display());
                                }
                                Err(errors) => {
                                    eprintln!("Assembly failed:");
                                    for error in errors {
                                        eprintln!("  - {}", error);
                                    }
                                    std::process::exit(1);
                                }
                            }
                        }
                        "macro" => {
                            let formatter = MacroFormatter::new();
                            let macro_output = formatter.format_full_program(
                                &obj.instructions,
                                Some(&obj.data),
                                None,
                                Some(&format!("Assembled from {}", input.display())),
                            );
                            fs::write(&output_path, macro_output)?;
                            println!("✓ Formatted to macro file {}", output_path.display());
                        }
                        _ => {
                            eprintln!("Unknown format: {}", format);
                            std::process::exit(1);
                        }
                    }
                    
                    // Print statistics
                    println!("  Instructions: {}", obj.instructions.len());
                    println!("  Labels: {}", obj.labels.len());
                    println!("  Data: {} bytes", obj.data.len());
                    if !obj.unresolved_references.is_empty() {
                        println!("  ⚠ Unresolved references: {}", obj.unresolved_references.len());
                        for (idx, unresolved) in &obj.unresolved_references {
                            println!("    - Instruction {}: {}", idx, unresolved.label);
                        }
                    }
                }
                Err(errors) => {
                    eprintln!("Assembly failed:");
                    for error in errors {
                        eprintln!("  - {}", error);
                    }
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Format { input, output } => {
            let content = fs::read_to_string(&input)?;
            let obj: ripple_asm::ObjectFile = serde_json::from_str(&content)?;
            
            let formatter = MacroFormatter::new();
            let macro_output = formatter.format_full_program(
                &obj.instructions,
                Some(&obj.data),
                None,
                Some(&format!("Formatted from {}", input.display())),
            );
            
            let output_path = output.unwrap_or_else(|| {
                let mut path = input.clone();
                path.set_extension("bfm");
                path
            });
            
            fs::write(&output_path, macro_output)?;
            println!("✓ Formatted to {}", output_path.display());
        }
        
        Commands::Check { input } => {
            let source = fs::read_to_string(&input)?;
            let assembler = RippleAssembler::new(AssemblerOptions::default());
            
            match assembler.assemble(&source) {
                Ok(obj) => {
                    println!("✓ {} is valid", input.display());
                    println!("  Instructions: {}", obj.instructions.len());
                    println!("  Labels: {}", obj.labels.len());
                    println!("  Data: {} bytes", obj.data.len());
                    
                    if !obj.unresolved_references.is_empty() {
                        println!("  ⚠ Unresolved references:");
                        for (_, unresolved) in &obj.unresolved_references {
                            println!("    - {}", unresolved.label);
                        }
                    }
                }
                Err(errors) => {
                    eprintln!("✗ {} has errors:", input.display());
                    for error in errors {
                        eprintln!("  - {}", error);
                    }
                    std::process::exit(1);
                }
            }
        }
    }
    
    Ok(())
}