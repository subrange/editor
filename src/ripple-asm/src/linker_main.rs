use clap::Parser;
use ripple_asm::{Linker, MacroFormatter};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rlink")]
#[command(about = "Ripple Linker - Links object files into executable")]
#[command(version)]
struct Cli {
    /// Input object files (.pobj)
    inputs: Vec<PathBuf>,
    
    /// Output file
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Output format (binary, text, macro)
    #[arg(short, long, default_value = "binary")]
    format: String,
    
    /// Bank size
    #[arg(short, long, default_value = "16")]
    bank_size: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    if cli.inputs.is_empty() {
        eprintln!("Error: No input files specified");
        std::process::exit(1);
    }
    
    // Load all object files
    let mut object_files = Vec::new();
    for input_path in &cli.inputs {
        let content = fs::read_to_string(input_path)?;
        match serde_json::from_str(&content) {
            Ok(obj) => {
                println!("Loaded {}", input_path.display());
                object_files.push(obj);
            }
            Err(e) => {
                eprintln!("Failed to parse {}: {}", input_path.display(), e);
                std::process::exit(1);
            }
        }
    }
    
    // Link the object files
    let linker = Linker::new(cli.bank_size);
    match linker.link(object_files) {
        Ok(program) => {
            let output_path = cli.output.unwrap_or_else(|| {
                PathBuf::from(match cli.format.as_str() {
                    "text" => "a.txt",
                    "macro" => "a.bfm",
                    _ => "a.out",
                })
            });
            
            match cli.format.as_str() {
                "binary" => {
                    let binary = program.to_binary();
                    fs::write(&output_path, binary)?;
                    println!("✓ Linked to binary {}", output_path.display());
                }
                "text" => {
                    let text = program.to_text();
                    fs::write(&output_path, text)?;
                    println!("✓ Linked to text {}", output_path.display());
                }
                "macro" => {
                    let formatter = MacroFormatter::new();
                    let macro_output = formatter.format_linked_program(&program);
                    fs::write(&output_path, macro_output)?;
                    println!("✓ Linked to macro file {}", output_path.display());
                }
                _ => {
                    eprintln!("Unknown format: {}", cli.format);
                    std::process::exit(1);
                }
            }
            
            // Print statistics
            println!("  Entry point: 0x{:08X}", program.entry_point);
            println!("  Instructions: {}", program.instructions.len());
            println!("  Data: {} bytes", program.data.len());
            println!("  Labels: {}", program.labels.len());
        }
        Err(errors) => {
            eprintln!("Linking failed:");
            for error in errors {
                eprintln!("  - {}", error);
            }
            std::process::exit(1);
        }
    }
    
    Ok(())
}