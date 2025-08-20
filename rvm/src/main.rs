mod constants;
mod debug;
mod tui_debugger;
mod vm;
mod debugger_ui;
mod mode;
mod settings;
mod display_rgb565;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use vm::VM;
use constants::{DEFAULT_BANK_SIZE, DEFAULT_MEMORY_SIZE};
use debug::Debugger;
use colored::*;
use crossterm::{terminal, cursor, style::ResetColor, ExecutableCommand};

/// Install signal handlers to ensure terminal cleanup on exit
fn install_signal_handlers() {
    use signal_hook::{consts::SIGTERM, consts::SIGINT, iterator::Signals};
    use std::thread;
    
    // Try to install signal handlers, but don't fail if it doesn't work
    // (e.g., on some platforms or in some environments)
    if let Ok(mut signals) = Signals::new(&[SIGINT, SIGTERM]) {
        thread::spawn(move || {
            for _ in signals.forever() {
                // Restore terminal before exiting
                let _ = terminal::disable_raw_mode();
                let _ = io::stderr().execute(cursor::Show);
                let _ = io::stderr().execute(terminal::LeaveAlternateScreen);
                let _ = io::stderr().execute(ResetColor);
                let _ = io::stderr().flush();
                
                // Exit the process
                process::exit(130); // 128 + SIGINT(2) = 130
            }
        });
    }
}

fn print_usage() {
    eprintln!("Usage: rvm [OPTIONS] <binary-file>");
    eprintln!();
    eprintln!("Run a Ripple VM binary program");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -b, --bank-size <size>   Set bank size (default: {DEFAULT_BANK_SIZE})");
    eprintln!("  -m, --memory <size>      Set memory size in words (default: {DEFAULT_MEMORY_SIZE})");
    eprintln!("  -f, --frequency <hz>     Set virtual CPU frequency (e.g., 1MHz, 500KHz, 2.5GHz)");
    eprintln!("  -s, --seed <value>       Set RNG seed (default: 0x12345678)");
    eprintln!("  -d, --debug              Enable debug mode (step through execution)");
    eprintln!("  -t, --tui                Enable TUI debugger_ui mode");
    eprintln!("  -v, --verbose            Show VM state during execution");
    eprintln!("  --visual                 Enable RGB565 visual display window");
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install terminal cleanup hooks
    vm::install_terminal_cleanup_hook();
    install_signal_handlers();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    let mut bank_size = DEFAULT_BANK_SIZE;
    let mut memory_size: Option<usize> = None;
    let mut frequency: Option<u64> = None;
    let mut rng_seed: Option<u32> = None;
    let mut debug_mode = false;
    let mut tui_mode = false;
    let mut verbose = false;
    let mut visual_mode = false;
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
            "-f" | "--frequency" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --frequency requires an argument");
                    process::exit(1);
                }
                i += 1;
                frequency = Some(parse_frequency(&args[i]).unwrap_or_else(|e| {
                    eprintln!("Error: Invalid frequency '{}': {}", args[i], e);
                    process::exit(1);
                }));
            },
            "-s" | "--seed" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --seed requires an argument");
                    process::exit(1);
                }
                i += 1;
                // Parse as hex if it starts with 0x, otherwise decimal
                let seed_str = &args[i];
                rng_seed = Some(if seed_str.starts_with("0x") || seed_str.starts_with("0X") {
                    u32::from_str_radix(&seed_str[2..], 16).unwrap_or_else(|_| {
                        eprintln!("Error: Invalid hex seed: {}", args[i]);
                        process::exit(1);
                    })
                } else {
                    seed_str.parse().unwrap_or_else(|_| {
                        eprintln!("Error: Invalid seed: {}", args[i]);
                        process::exit(1);
                    })
                });
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
            "--visual" => {
                visual_mode = true;
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
        eprintln!("Error reading file '{file_path}': {e}");
        process::exit(1);
    });
    
    // Create and initialize the VM
    let mut vm = if let Some(mem_size) = memory_size {
        VM::with_memory_size(bank_size, mem_size)
    } else {
        VM::new(bank_size) // Default 64K memory
    };
    
    // Set RNG seed if specified
    if let Some(seed) = rng_seed {
        vm.set_rng_seed(seed);
        if verbose {
            println!("RNG seed set to: 0x{seed:08X}");
        }
    }
    
    // Set verbose mode if requested
    vm.verbose = verbose;
    
    // Load the binary
    if let Err(e) = vm.load_binary(&binary) {
        eprintln!("Error loading binary: {e}");
        process::exit(1);
    }
    
    if verbose {
        println!("Loading binary from {file_path}...");
        println!("Bank size: {bank_size}");
        println!("Memory size: {} words", vm.memory.len());
    }
    
    if verbose {
        println!("Loaded {} instructions", vm.instructions.len());
        println!("Starting execution at PC={}, PCB={}", vm.registers[1], vm.registers[2]);
        println!();
    }
    
    // Run the VM
    let vm_moved_to_arc = if visual_mode {
        // Visual mode: run VM in background thread, display on main thread
        eprintln!("Starting in visual mode...");
        
        // Initialize a default RGB565 display that will be configured when the program runs
        if vm.rgb565_display.is_none() {
            vm.rgb565_display = Some(display_rgb565::RGB565Display::new());
        }
        
        // Get the display state
        let display_state = vm.rgb565_display.as_ref().unwrap().get_state();
        
        // Run VM in a background thread
        let vm = Arc::new(Mutex::new(vm));
        let vm_clone = Arc::clone(&vm);
        let frequency_clone = frequency;
        
        let vm_thread = thread::spawn(move || {
            let mut vm = vm_clone.lock().unwrap();
            
            // Run with frequency limiting if specified
            if let Some(freq) = frequency_clone {
                if let Err(e) = run_with_frequency(&mut vm, freq) {
                    eprintln!("Runtime error: {}", e);
                }
            } else {
                if let Err(e) = vm.run() {
                    eprintln!("Runtime error: {}", e);
                }
            }
            
            eprintln!("VM execution completed. Close the window to exit.");
            // Don't shutdown display immediately - let user close the window
        });
        
        // Run display on main thread
        if let Err(e) = display_rgb565::run_rgb565_display(display_state) {
            eprintln!("Display error: {}", e);
        }
        
        // Wait for VM thread to finish
        let _ = vm_thread.join();
        true // VM was moved to Arc
    } else if tui_mode {
        // Use the TUI debugger_ui
        vm.debug_mode = true;
        let mut tui = tui_debugger::TuiDebugger::new();
        if let Err(e) = tui.run(&mut vm) {
            eprintln!("TUI error: {e}");
            process::exit(1);
        }
        false // VM not moved to Arc
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
                        eprintln!("Runtime error: {e}");
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
        false // VM not moved to Arc
    } else {
        // Run normally with optional frequency limiting
        if let Some(freq) = frequency {
            if verbose {
                println!("Running at {freq} Hz");
            }
            run_with_frequency(&mut vm, freq)?;
        } else if let Err(e) = vm.run() {
            eprintln!("Runtime error: {e}");
            process::exit(1);
        }
        false // VM not moved to Arc
    };
    
    // Output is now printed in real-time during execution
    if verbose && !vm_moved_to_arc {
        println!();
        println!("Execution completed");
    }
    
    // Explicitly ensure terminal is restored before exit
    // This handles cases where Drop might not be called
    let _ = terminal::disable_raw_mode();
    let _ = io::stderr().execute(cursor::Show);
    let _ = io::stderr().execute(terminal::LeaveAlternateScreen);
    let _ = io::stderr().execute(ResetColor);
    let _ = io::stderr().flush();
    
    Ok(())
}

/// Parse frequency from string (e.g., "1MHz", "500KHz", "1000000", "2.5MHz")
fn parse_frequency(s: &str) -> Result<u64, String> {
    let s = s.trim();
    
    // Check for suffix
    if let Some(num_str) = s.strip_suffix("GHz") {
        parse_float_with_multiplier(num_str, 1_000_000_000)
    } else if let Some(num_str) = s.strip_suffix("MHz") {
        parse_float_with_multiplier(num_str, 1_000_000)
    } else if let Some(num_str) = s.strip_suffix("KHz") {
        parse_float_with_multiplier(num_str, 1_000)
    } else if let Some(num_str) = s.strip_suffix("kHz") {
        parse_float_with_multiplier(num_str, 1_000)
    } else if let Some(num_str) = s.strip_suffix("Hz") {
        parse_float_with_multiplier(num_str, 1)
    } else {
        // Try to parse as plain number (assumed to be Hz)
        s.parse::<u64>().map_err(|_| format!("Invalid frequency value: {s}"))
    }
}

fn parse_float_with_multiplier(s: &str, multiplier: u64) -> Result<u64, String> {
    let s = s.trim();
    if let Ok(f) = s.parse::<f64>() {
        Ok((f * multiplier as f64) as u64)
    } else {
        Err(format!("Invalid numeric value: {s}"))
    }
}

/// Run VM with frequency limiting
fn run_with_frequency(vm: &mut VM, frequency: u64) -> Result<(), String> {
    // Target 60 FPS for smooth animation
    const TARGET_FPS: u64 = 60;
    const NANOS_PER_SECOND: u64 = 1_000_000_000;
    
    // Calculate instructions per frame
    let instructions_per_frame = frequency / TARGET_FPS;
    let frame_duration = Duration::from_nanos(NANOS_PER_SECOND / TARGET_FPS);
    
    let mut last_frame_time = Instant::now();
    let mut instructions_in_frame = 0;
    
    while matches!(vm.state, vm::VMState::Running) {
        // Execute one instruction
        vm.step()?;
        instructions_in_frame += 1;
        
        // Check if we've executed enough instructions for this frame
        if instructions_in_frame >= instructions_per_frame {
            // Wait for the rest of the frame duration
            let elapsed = last_frame_time.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
            
            // Reset for next frame
            last_frame_time = Instant::now();
            instructions_in_frame = 0;
        }
        
        // Stop if we hit a breakpoint in debug mode
        if matches!(vm.state, vm::VMState::Breakpoint) {
            break;
        }
    }
    
    Ok(())
}