use bf_macro_expander::{create_macro_expander, MacroExpanderOptions};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "bfm")]
#[command(author = "BF Macro Expander")]
#[command(version = "0.1.0")]
#[command(about = "Brainfuck macro expander", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Expand macros in a file
    Expand {
        /// Input file path (use - for stdin)
        input: String,
        
        /// Output file path (use - for stdout)
        #[arg(short, long, default_value = "-")]
        output: String,
        
        /// Strip comments from output
        #[arg(long, default_value_t = true)]
        strip_comments: bool,
        
        /// Collapse empty lines in output
        #[arg(long)]
        collapse_empty_lines: bool,
        
        /// Generate source map
        #[arg(long)]
        source_map: bool,
        
        /// Enable circular dependency detection
        #[arg(long)]
        circular_check: bool,
        
        /// Output format
        #[arg(long, value_enum, default_value = "code")]
        format: OutputFormat,
    },
    
    /// List macros defined in a file
    List {
        /// Input file path (use - for stdin)
        input: String,
        
        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: ListFormat,
    },
    
    /// Validate macro definitions
    Validate {
        /// Input file path (use - for stdin)
        input: String,
        
        /// Strict mode (fail on warnings)
        #[arg(long)]
        strict: bool,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    /// Output only the expanded code
    Code,
    /// Output JSON with code, errors, and metadata
    Json,
    /// Output detailed debug information
    Debug,
}

#[derive(clap::ValueEnum, Clone)]
enum ListFormat {
    /// Simple text format
    Text,
    /// JSON format
    Json,
    /// Detailed format with parameters and body
    Detailed,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Expand { 
            input, 
            output, 
            strip_comments, 
            collapse_empty_lines,
            source_map,
            circular_check,
            format,
        } => {
            let content = read_input(&input)?;
            
            let options = MacroExpanderOptions {
                strip_comments,
                collapse_empty_lines,
                generate_source_map: source_map,
                enable_circular_dependency_detection: circular_check,
            };
            
            let mut expander = create_macro_expander();
            let result = expander.expand(&content, options);
            
            // Check for errors
            if !result.errors.is_empty() {
                eprintln!("Expansion completed with {} error(s):", result.errors.len());
                for error in &result.errors {
                    eprintln!("  [{:?}] {}", error.error_type, error.message);
                    if let Some(loc) = &error.location {
                        eprintln!("    at line {}, column {}", loc.line + 1, loc.column + 1);
                    }
                }
                if matches!(format, OutputFormat::Code) {
                    return Err("Expansion failed with errors".into());
                }
            }
            
            let output_content = match format {
                OutputFormat::Code => result.expanded,
                OutputFormat::Json => serde_json::to_string_pretty(&result)?,
                OutputFormat::Debug => {
                    format!(
                        "=== EXPANDED CODE ===\n{}\n\n\
                         === MACROS ({}) ===\n{}\n\n\
                         === ERRORS ({}) ===\n{}\n\n\
                         === TOKENS ({}) ===\n{}",
                        result.expanded,
                        result.macros.len(),
                        result.macros.iter()
                            .map(|m| format!("  {} ({} params)", m.name, m.parameters.as_ref().map(|p| p.len()).unwrap_or(0)))
                            .collect::<Vec<_>>()
                            .join("\n"),
                        result.errors.len(),
                        result.errors.iter()
                            .map(|e| format!("  [{:?}] {}", e.error_type, e.message))
                            .collect::<Vec<_>>()
                            .join("\n"),
                        result.tokens.len(),
                        result.tokens.iter()
                            .map(|t| format!("  {:?} '{}' [{}-{}]", t.token_type, t.name, t.range.start, t.range.end))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                }
            };
            
            write_output(&output, &output_content)?;
            
            if source_map && result.source_map.is_some() {
                let map_path = if output == "-" {
                    "output.map.json".to_string()
                } else {
                    format!("{}.map", output)
                };
                let map_json = serde_json::to_string_pretty(&result.source_map)?;
                fs::write(&map_path, map_json)?;
                eprintln!("Source map written to: {}", map_path);
            }
        }
        
        Commands::List { input, format } => {
            let content = read_input(&input)?;
            
            let mut expander = create_macro_expander();
            let result = expander.expand(&content, MacroExpanderOptions::default());
            
            match format {
                ListFormat::Text => {
                    println!("Found {} macro(s):", result.macros.len());
                    for macro_def in &result.macros {
                        let params = macro_def.parameters.as_ref()
                            .map(|p| format!("({})", p.join(", ")))
                            .unwrap_or_default();
                        println!("  {}{}", macro_def.name, params);
                    }
                }
                ListFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result.macros)?);
                }
                ListFormat::Detailed => {
                    for macro_def in &result.macros {
                        println!("Macro: {}", macro_def.name);
                        if let Some(params) = &macro_def.parameters {
                            println!("  Parameters: {}", params.join(", "));
                        }
                        println!("  Location: line {}, column {}", 
                            macro_def.source_location.line + 1,
                            macro_def.source_location.column + 1);
                        println!("  Body: {}", 
                            if macro_def.body.len() > 50 {
                                format!("{}...", &macro_def.body[..50])
                            } else {
                                macro_def.body.clone()
                            });
                        println!();
                    }
                }
            }
        }
        
        Commands::Validate { input, strict } => {
            let content = read_input(&input)?;
            
            let options = MacroExpanderOptions {
                strip_comments: false,
                collapse_empty_lines: false,
                generate_source_map: false,
                enable_circular_dependency_detection: true,
            };
            
            let mut expander = create_macro_expander();
            let result = expander.expand(&content, options);
            
            let error_count = result.errors.len();
            let warning_count = 0; // Could be extended to include warnings
            
            if error_count == 0 && warning_count == 0 {
                println!("✓ No issues found");
                return Ok(());
            }
            
            if error_count > 0 {
                println!("✗ Found {} error(s):", error_count);
                for error in &result.errors {
                    println!("  [{:?}] {}", error.error_type, error.message);
                    if let Some(loc) = &error.location {
                        println!("    at line {}, column {}", loc.line + 1, loc.column + 1);
                    }
                }
            }
            
            if warning_count > 0 {
                println!("⚠ Found {} warning(s)", warning_count);
            }
            
            if error_count > 0 || (strict && warning_count > 0) {
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}

fn read_input(path: &str) -> io::Result<String> {
    if path == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    } else {
        fs::read_to_string(path)
    }
}

fn write_output(path: &str, content: &str) -> io::Result<()> {
    if path == "-" {
        print!("{}", content);
    } else {
        fs::write(path, content)?;
    }
    Ok(())
}