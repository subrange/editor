mod constants;
mod debug;
mod tui_debugger;
mod vm;
mod debugger_ui;
mod mode;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;
use vm::VM;
use constants::{DEFAULT_BANK_SIZE, DEFAULT_MEMORY_SIZE};
use debug::Debugger;
use colored::*;

fn print_usage() {
    eprintln!("Usage: rvm [OPTIONS] <binary-file>");
    eprintln!();
    eprintln!("Run a Ripple VM binary program");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -b, --bank-size <size>   Set bank size (default: {})", DEFAULT_BANK_SIZE);
    eprintln!("  -m, --memory <size>      Set memory size in words (default: {})", DEFAULT_MEMORY_SIZE);
    eprintln!("  -d, --debug              Enable debug mode (step through execution)");
    eprintln!("  -t, --tui                Enable TUI debugger_ui mode");
    eprintln!("  -v, --verbose            Show VM state during execution");
    eprintln!("  -h, --help               Show this help message");
    eprintln!();
    eprintln!("TUI DEBUGGER MODE (-t):");
    eprintln!("  Professional terminal-based debugger_ui with multiple panes:");
    eprintln!("  - Disassembly view with breakpoints and execution tracking");
    eprintln!("  - Register display with change highlighting");
    eprintln!("  - Memory viewer with hex/ASCII display and editing");
    eprintln!("  - Stack trace and memory watches");
    eprintln!("  - Output buffer display");
    eprintln!();
    eprintln!("  Key features:");
    eprintln!("  • Set breakpoints at cursor or by instruction number");
    eprintln!("  • Step, run, and continue execution");
    eprintln!("  • Edit memory and registers on-the-fly");
    eprintln!("  • Navigate with vim-style keys or arrows");
    eprintln!("  • Command mode for advanced operations");
    eprintln!("  • Press '?' in TUI for complete keyboard shortcuts");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    let mut bank_size = DEFAULT_BANK_SIZE;
    let mut memory_size: Option<usize> = None;
    let mut debug_mode = false;
    let mut tui_mode = false;
    let mut verbose = false;
    let mut file_path = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            },
            "-b" | "--bank-size" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --bank-size requires an argument");
                    process::exit(1);
                }
                i += 1;
                bank_size = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("Error: Invalid bank size: {}", args[i]);
                    process::exit(1);
                });
            },
            "-m" | "--memory" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --memory requires an argument");
                    process::exit(1);
                }
                i += 1;
                memory_size = Some(args[i].parse().unwrap_or_else(|_| {
                    eprintln!("Error: Invalid memory size: {}", args[i]);
                    process::exit(1);
                }));
            },
            "-d" | "--debug" => {
                debug_mode = true;
            },
            "-t" | "--tui" => {
                tui_mode = true;
            },
            "-v" | "--verbose" => {
                verbose = true;
            },
            _ => {
                if args[i].starts_with('-') {
                    eprintln!("Error: Unknown option: {}", args[i]);
                    process::exit(1);
                }
                file_path = Some(args[i].clone());
            }
        }
        i += 1;
    }
    
    let file_path = file_path.unwrap_or_else(|| {
        eprintln!("Error: No input file specified");
        print_usage();
        process::exit(1);
    });
    
    // Read the binary file
    let binary = fs::read(&file_path).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", file_path, e);
        process::exit(1);
    });
    
    // Create and initialize the VM
    let mut vm = if let Some(mem_size) = memory_size {
        VM::with_memory_size(bank_size, mem_size)
    } else {
        VM::new(bank_size) // Default 64K memory
    };
    
    // Load the binary
    if let Err(e) = vm.load_binary(&binary) {
        eprintln!("Error loading binary: {}", e);
        process::exit(1);
    }
    
    if verbose {
        println!("Loading binary from {}...", file_path);
        println!("Bank size: {}", bank_size);
        println!("Memory size: {} words", vm.memory.len());
    }
    
    if verbose {
        println!("Loaded {} instructions", vm.instructions.len());
        println!("Starting execution at PC={}, PCB={}", vm.registers[1], vm.registers[2]);
        println!();
    }
    
    // Run the VM
    if tui_mode {
        // Use the TUI debugger_ui
        vm.debug_mode = true;
        let mut tui = tui_debugger::TuiDebugger::new();
        if let Err(e) = tui.run(&mut vm) {
            eprintln!("TUI error: {}", e);
            process::exit(1);
        }
    } else if debug_mode {
        vm.debug_mode = true;  // Enable debug mode in VM
        Debugger::print_welcome();
        
        let stdin = io::stdin();
        let mut input = String::new();
        
        // Show initial state
        let debugger = Debugger::new();
        debugger.print_state(&vm);
        
        loop {
            
            // Check VM state
            match vm.state {
                vm::VMState::Halted => {
                    println!("\n{}", "Program halted".bright_red().bold());
                    break;
                }
                vm::VMState::Breakpoint => {
                    println!("\n{}", ">>> Breakpoint hit <<<".bright_yellow().bold());
                    println!("{}", "Press 'c' to continue, Enter to step".bright_black());
                }
                _ => {}
            }
            
            // Wait for input with colored prompt
            print!("{} ", ">".bright_green().bold());
            io::stdout().flush().unwrap();
            input.clear();
            stdin.read_line(&mut input).unwrap();
            
            match input.trim() {
                "q" => break,
                "r" => {
                    // Run to completion
                    if let Err(e) = vm.run() {
                        eprintln!("Runtime error: {}", e);
                        process::exit(1);
                    }
                    break;
                },
                "c" if matches!(vm.state, vm::VMState::Breakpoint) => {
                    // Continue from breakpoint.rs
                    vm.state = vm::VMState::Running;
                    // Continue stepping
                },
                _ => {
                    // Step one instruction
                    if let Err(e) = vm.step() {
                        eprintln!("{}: {}", "Runtime error".bright_red().bold(), e);
                        process::exit(1);
                    }
                    
                    // Check for output
                    let output = vm.get_output();
                    if !output.is_empty() {
                        println!("\n{}: {}", 
                            "Output".bright_cyan().bold(),
                            String::from_utf8_lossy(&output)
                        );
                    }
                    
                    // Show new state
                    debugger.print_state(&vm);
                }
            }
        }
    } else {
        // Run normally
        if let Err(e) = vm.run() {
            eprintln!("Runtime error: {}", e);
            process::exit(1);
        }
    }
    
    // Get and print output
    let output = vm.get_output();
    if !output.is_empty() {
        io::stdout().write_all(&output).unwrap();
        io::stdout().flush().unwrap();
        
        // Ensure we end with a newline if the output didn't include one
        if !output.ends_with(&[b'\n']) {
            println!();
        }
    }
    
    if verbose {
        println!();
        println!("Execution completed");
        println!("Final state: {:?}", vm.state);
    }
}