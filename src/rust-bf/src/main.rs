use clap::{Parser, ArgAction};
use std::fs;
use std::io::{self, Read, Write};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "bf")]
#[command(about = "Optimizing Brainfuck interpreter", long_about = None)]
struct Args {
    /// Input Brainfuck file
    file: String,

    /// Cell size in bits (8, 16, or 32)
    #[arg(long, value_name = "BITS", default_value = "8")]
    cell_size: u8,

    /// Disable cell wrapping behavior
    #[arg(long = "no-wrap", action = ArgAction::SetFalse, default_value_t = true)]
    wrap: bool,

    /// Disable tape pointer wrapping behavior  
    #[arg(long = "no-wrap-tape", action = ArgAction::SetFalse, default_value_t = true)]
    wrap_tape: bool,

    /// Tape size (number of cells)
    #[arg(long, value_name = "SIZE", default_value = "30000")]
    tape_size: usize,

    /// Show execution statistics
    #[arg(long)]
    stat: bool,
}

#[derive(Debug, Clone)]
enum Op {
    Add(i32),
    Move(i32),
    Loop(Vec<Op>),
    Input,
    Output,
    Clear,
    MulAdd(i32, i32), // [>+++<-] becomes MulAdd(1, 3)
    ScanLeft(i32),
    ScanRight(i32),
}

trait Cell: Copy + Default + PartialEq {
    fn from_u8(v: u8) -> Self;
    fn to_u8(self) -> u8;
    fn wrapping_add(self, n: i32) -> Self;
    fn saturating_add(self, n: i32) -> Self;
}

impl Cell for u8 {
    fn from_u8(v: u8) -> Self {
        v
    }
    
    fn to_u8(self) -> u8 {
        self
    }
    
    fn wrapping_add(self, n: i32) -> Self {
        if n >= 0 {
            self.wrapping_add(n as u8)
        } else {
            self.wrapping_sub((-n) as u8)
        }
    }
    
    fn saturating_add(self, n: i32) -> Self {
        if n >= 0 {
            self.saturating_add(n as u8)
        } else {
            self.saturating_sub((-n) as u8)
        }
    }
}

impl Cell for u16 {
    fn from_u8(v: u8) -> Self {
        v as u16
    }
    
    fn to_u8(self) -> u8 {
        (self & 0xFF) as u8
    }
    
    fn wrapping_add(self, n: i32) -> Self {
        if n >= 0 {
            self.wrapping_add(n as u16)
        } else {
            self.wrapping_sub((-n) as u16)
        }
    }
    
    fn saturating_add(self, n: i32) -> Self {
        if n >= 0 {
            self.saturating_add(n as u16)
        } else {
            self.saturating_sub((-n) as u16)
        }
    }
}

impl Cell for u32 {
    fn from_u8(v: u8) -> Self {
        v as u32
    }
    
    fn to_u8(self) -> u8 {
        (self & 0xFF) as u8
    }
    
    fn wrapping_add(self, n: i32) -> Self {
        if n >= 0 {
            self.wrapping_add(n as u32)
        } else {
            self.wrapping_sub((-n) as u32)
        }
    }
    
    fn saturating_add(self, n: i32) -> Self {
        if n >= 0 {
            self.saturating_add(n as u32)
        } else {
            self.saturating_sub((-n) as u32)
        }
    }
}

struct Interpreter<T: Cell> {
    tape: Vec<T>,
    ptr: usize,
    wrap: bool,
    wrap_tape: bool,
}

impl<T: Cell> Interpreter<T> {
    fn new(tape_size: usize, wrap: bool, wrap_tape: bool) -> Self {
        Self {
            tape: vec![T::default(); tape_size],
            ptr: 0,
            wrap,
            wrap_tape,
        }
    }

    fn run(&mut self, ops: &[Op]) {
        for op in ops {
            self.execute(op);
        }
    }

    fn execute(&mut self, op: &Op) {
        match op {
            Op::Add(n) => {
                if self.wrap {
                    self.tape[self.ptr] = self.tape[self.ptr].wrapping_add(*n);
                } else {
                    self.tape[self.ptr] = self.tape[self.ptr].saturating_add(*n);
                }
            }
            Op::Move(n) => {
                let new_ptr = (self.ptr as i64) + (*n as i64);
                if self.wrap_tape {
                    // Wrap around the tape
                    if new_ptr < 0 {
                        self.ptr = ((new_ptr % self.tape.len() as i64 + self.tape.len() as i64) % self.tape.len() as i64) as usize;
                    } else {
                        self.ptr = (new_ptr as usize) % self.tape.len();
                    }
                } else {
                    // Clamp to tape bounds (no wrapping)
                    if new_ptr < 0 {
                        self.ptr = 0;
                    } else if new_ptr >= self.tape.len() as i64 {
                        self.ptr = self.tape.len() - 1;
                    } else {
                        self.ptr = new_ptr as usize;
                    }
                }
            }
            Op::Loop(ops) => {
                while self.tape[self.ptr] != T::default() {
                    self.run(ops);
                }
            }
            Op::Input => {
                let mut buf = [0];
                io::stdin().read_exact(&mut buf).unwrap_or(());
                self.tape[self.ptr] = T::from_u8(buf[0]);
            }
            Op::Output => {
                print!("{}", self.tape[self.ptr].to_u8() as char);
                io::stdout().flush().unwrap();
            }
            Op::Clear => {
                self.tape[self.ptr] = T::default();
            }
            Op::MulAdd(offset, factor) => {
                let val = self.tape[self.ptr];
                if val != T::default() {
                    let target = (self.ptr as i64) + (*offset as i64);
                    let target_ptr = if self.wrap_tape {
                        if target < 0 {
                            ((target % self.tape.len() as i64 + self.tape.len() as i64) % self.tape.len() as i64) as usize
                        } else {
                            (target as usize) % self.tape.len()
                        }
                    } else {
                        if target < 0 {
                            0
                        } else if target >= self.tape.len() as i64 {
                            self.tape.len() - 1
                        } else {
                            target as usize
                        }
                    };
                    
                    // MulAdd needs to work with the actual cell value, not just u8
                    let result = match std::mem::size_of::<T>() {
                        1 => (val.to_u8() as i32 * factor),
                        2 => {
                            let v = unsafe { std::mem::transmute_copy::<T, u16>(&val) };
                            (v as i32 * factor)
                        },
                        4 => {
                            let v = unsafe { std::mem::transmute_copy::<T, u32>(&val) };
                            // Need to handle potential overflow for 32-bit multiplication
                            ((v as i64 * *factor as i64) & 0xFFFFFFFF) as i32
                        },
                        _ => unreachable!()
                    };
                    
                    if self.wrap {
                        self.tape[target_ptr] = self.tape[target_ptr].wrapping_add(result);
                    } else {
                        self.tape[target_ptr] = self.tape[target_ptr].saturating_add(result);
                    }
                    self.tape[self.ptr] = T::default();
                }
            }
            Op::ScanLeft(step) => {
                while self.tape[self.ptr] != T::default() {
                    let new_ptr = (self.ptr as i64) - (step.abs() as i64);
                    if self.wrap_tape {
                        if new_ptr < 0 {
                            self.ptr = ((new_ptr % self.tape.len() as i64 + self.tape.len() as i64) % self.tape.len() as i64) as usize;
                        } else {
                            self.ptr = new_ptr as usize;
                        }
                    } else {
                        if new_ptr < 0 {
                            self.ptr = 0;
                            break; // Stop at boundary
                        } else {
                            self.ptr = new_ptr as usize;
                        }
                    }
                    if self.ptr == 0 && !self.wrap_tape {
                        break; // Stop at the beginning if not wrapping
                    }
                }
            }
            Op::ScanRight(step) => {
                while self.tape[self.ptr] != T::default() {
                    let new_ptr = (self.ptr as i64) + (*step as i64);
                    if self.wrap_tape {
                        self.ptr = (new_ptr as usize) % self.tape.len();
                    } else {
                        if new_ptr >= self.tape.len() as i64 {
                            self.ptr = self.tape.len() - 1;
                            break; // Stop at boundary
                        } else {
                            self.ptr = new_ptr as usize;
                        }
                    }
                }
            }
        }
    }
}

fn parse(code: &str) -> Vec<Op> {
    let mut ops = Vec::new();
    let mut chars = code.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '+' => {
                let mut count = 1;
                while chars.peek() == Some(&'+') {
                    chars.next();
                    count += 1;
                }
                ops.push(Op::Add(count));
            }
            '-' => {
                let mut count = -1;
                while chars.peek() == Some(&'-') {
                    chars.next();
                    count -= 1;
                }
                ops.push(Op::Add(count));
            }
            '>' => {
                let mut count = 1;
                while chars.peek() == Some(&'>') {
                    chars.next();
                    count += 1;
                }
                ops.push(Op::Move(count));
            }
            '<' => {
                let mut count = -1;
                while chars.peek() == Some(&'<') {
                    chars.next();
                    count -= 1;
                }
                ops.push(Op::Move(count));
            }
            '[' => {
                let mut depth = 1;
                let mut loop_code = String::new();
                
                while depth > 0 {
                    match chars.next() {
                        Some('[') => {
                            loop_code.push('[');
                            depth += 1;
                        }
                        Some(']') => {
                            depth -= 1;
                            if depth > 0 {
                                loop_code.push(']');
                            }
                        }
                        Some(c) => loop_code.push(c),
                        None => break,
                    }
                }
                
                let loop_ops = parse(&loop_code);
                ops.push(Op::Loop(loop_ops));
            }
            ',' => ops.push(Op::Input),
            '.' => ops.push(Op::Output),
            _ => {}
        }
    }
    
    ops
}

fn optimize(ops: Vec<Op>) -> Vec<Op> {
    let mut result = Vec::new();
    
    for op in ops {
        match op {
            Op::Loop(loop_ops) => {
                let optimized_loop = optimize(loop_ops);
                
                // Pattern: [-] or [+] -> Clear
                if optimized_loop.len() == 1 {
                    if let Op::Add(n) = optimized_loop[0] {
                        if n == -1 || n == 1 {
                            result.push(Op::Clear);
                            continue;
                        }
                    }
                }
                
                // Pattern: [>] or [>>...] -> ScanRight
                if optimized_loop.len() == 1 {
                    if let Op::Move(n) = optimized_loop[0] {
                        if n > 0 {
                            result.push(Op::ScanRight(n));
                            continue;
                        } else if n < 0 {
                            result.push(Op::ScanLeft(n));
                            continue;
                        }
                    }
                }
                
                // Pattern: [->+<] or [->+++<] etc. -> MulAdd
                if optimized_loop.len() == 4 {
                    if let (Op::Add(-1), Op::Move(m1), Op::Add(factor), Op::Move(m2)) = 
                        (&optimized_loop[0], &optimized_loop[1], &optimized_loop[2], &optimized_loop[3]) {
                        if m1 + m2 == 0 && *m1 != 0 {
                            result.push(Op::MulAdd(*m1, *factor));
                            continue;
                        }
                    }
                }
                
                // Pattern: [->>+++<<] etc. -> MulAdd with multiple cells
                if optimized_loop.len() >= 3 && optimized_loop.len() % 2 == 0 {
                    if let Op::Add(-1) = optimized_loop[0] {
                        let mut valid_muladd = true;
                        let mut total_move = 0;
                        let mut muladds = Vec::new();
                        
                        for i in (1..optimized_loop.len()).step_by(2) {
                            if i + 1 < optimized_loop.len() {
                                if let (Op::Move(m), Op::Add(a)) = (&optimized_loop[i], &optimized_loop[i + 1]) {
                                    total_move += m;
                                    muladds.push((total_move, *a));
                                } else {
                                    valid_muladd = false;
                                    break;
                                }
                            }
                        }
                        
                        if valid_muladd && total_move == 0 && muladds.len() == 1 {
                            let (offset, factor) = muladds[0];
                            result.push(Op::MulAdd(offset, factor));
                            continue;
                        }
                    }
                }
                
                result.push(Op::Loop(optimized_loop));
            }
            other => result.push(other),
        }
    }
    
    result
}

fn run_interpreter<T: Cell>(tape_size: usize, wrap: bool, wrap_tape: bool, optimized: &[Op]) {
    let mut interpreter = Interpreter::<T>::new(tape_size, wrap, wrap_tape);
    interpreter.run(optimized);
}

fn main() {
    let args = Args::parse();
    
    // Validate cell size
    match args.cell_size {
        8 | 16 | 32 => {}
        _ => {
            eprintln!("Error: cell-size must be 8, 16, or 32");
            std::process::exit(1);
        }
    }
    
    let code = match fs::read_to_string(&args.file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", args.file, e);
            std::process::exit(1);
        }
    };
    
    let start = if args.stat {
        Some(Instant::now())
    } else {
        None
    };
    
    let ops = parse(&code);
    let optimized = optimize(ops);
    
    // Run with appropriate cell size
    match args.cell_size {
        8 => run_interpreter::<u8>(args.tape_size, args.wrap, args.wrap_tape, &optimized),
        16 => run_interpreter::<u16>(args.tape_size, args.wrap, args.wrap_tape, &optimized),
        32 => run_interpreter::<u32>(args.tape_size, args.wrap, args.wrap_tape, &optimized),
        _ => unreachable!(),
    }
    
    if let Some(start_time) = start {
        let duration = start_time.elapsed();
        eprintln!("\nExecution time: {:.6} seconds", duration.as_secs_f64());
    }
}