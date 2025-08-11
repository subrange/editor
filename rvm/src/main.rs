mod vm;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;
use vm::VM;

fn print_usage() {
    eprintln!("Usage: rvm [OPTIONS] <binary-file>");
    eprintln!();
    eprintln!("Run a Ripple VM binary program");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -b, --bank-size <size>   Set bank size (default: 4096)");
    eprintln!("  -d, --debug              Enable debug mode (step through execution)");
    eprintln!("  -v, --verbose            Show VM state during execution");
    eprintln!("  -h, --help               Show this help message");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    let mut bank_size = 4096u16;
    let mut debug_mode = false;
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
            "-d" | "--debug" => {
                debug_mode = true;
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
    let mut vm = VM::new(bank_size);
    
    if verbose {
        println!("Loading binary from {}...", file_path);
        println!("Bank size: {}", bank_size);
    }
    
    // Load the binary
    if let Err(e) = vm.load_binary(&binary) {
        eprintln!("Error loading binary: {}", e);
        process::exit(1);
    }
    
    if verbose {
        println!("Loaded {} instructions", vm.instructions.len());
        println!("Starting execution at PC={}, PCB={}", vm.registers[1], vm.registers[2]);
        println!();
    }
    
    // Run the VM
    if debug_mode {
        println!("Debug mode - press Enter to step, 'r' to run, 'q' to quit");
        let stdin = io::stdin();
        let mut input = String::new();
        
        loop {
            // Print current state
            println!("PC={:04X} PCB={:04X}", vm.registers[1], vm.registers[2]);
            print!("Registers: ");
            for i in 0..18 {
                if i > 0 { print!(", "); }
                print!("R{}={:04X}", i, vm.registers[i]);
            }
            println!();
            
            // Get current instruction
            let pc = vm.registers[1] as usize;
            let pcb = vm.registers[2] as usize;
            let idx = pcb * bank_size as usize + pc;
            if idx < vm.instructions.len() {
                let instr = vm.instructions[idx];
                println!("Next: opcode={:02X} operands=[{:04X}, {:04X}, {:04X}]", 
                         instr.opcode, instr.word1, instr.word2, instr.word3);
            }
            
            // Check if halted
            if matches!(vm.state, vm::VMState::Halted) {
                println!("Program halted");
                break;
            }
            
            // Wait for input
            print!("> ");
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
                _ => {
                    // Step one instruction
                    if let Err(e) = vm.step() {
                        eprintln!("Runtime error: {}", e);
                        process::exit(1);
                    }
                    
                    // Check for output
                    let output = vm.get_output();
                    if !output.is_empty() {
                        print!("Output: ");
                        io::stdout().write_all(&output).unwrap();
                        io::stdout().flush().unwrap();
                    }
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
    }
    
    if verbose {
        println!();
        println!("Execution completed");
        println!("Final state: {:?}", vm.state);
    }
}