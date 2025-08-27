use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub enum Op {
    Add(i32),
    Move(i32),
    Loop(Vec<Op>),
    Input,
    Output,
    Clear,
    Set(i32),
    MulAdd(i32, i32),
    ScanLeft(i32),
    ScanRight(i32),
}

pub trait Cell: Copy + Default + PartialEq {
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

pub struct Interpreter<T: Cell> {
    pub tape: Vec<T>,
    pub ptr: usize,
    pub wrap: bool,
    pub wrap_tape: bool,
    pub output: Vec<u8>,
    pub input: Vec<u8>,
    pub input_ptr: usize,
}

pub struct InterpreterWithCallback<'a, T: Cell> {
    pub tape: Vec<T>,
    pub ptr: usize,
    pub wrap: bool,
    pub wrap_tape: bool,
    pub output: Vec<u8>,
    pub input: Vec<u8>,
    pub input_ptr: usize,
    pub output_callback: &'a js_sys::Function,
}

impl<T: Cell> Interpreter<T> {
    pub fn new(tape_size: usize, wrap: bool, wrap_tape: bool) -> Self {
        Self {
            tape: vec![T::default(); tape_size],
            ptr: 0,
            wrap,
            wrap_tape,
            output: Vec::new(),
            input: Vec::new(),
            input_ptr: 0,
        }
    }

    pub fn set_input(&mut self, input: Vec<u8>) {
        self.input = input;
        self.input_ptr = 0;
    }

    pub fn run(&mut self, ops: &[Op]) {
        for op in ops {
            self.execute(op);
        }
    }

    pub fn execute(&mut self, op: &Op) {
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
                    if new_ptr < 0 {
                        self.ptr = ((new_ptr % self.tape.len() as i64 + self.tape.len() as i64) % self.tape.len() as i64) as usize;
                    } else {
                        self.ptr = (new_ptr as usize) % self.tape.len();
                    }
                } else {
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
                if self.input_ptr < self.input.len() {
                    self.tape[self.ptr] = T::from_u8(self.input[self.input_ptr]);
                    self.input_ptr += 1;
                } else {
                    self.tape[self.ptr] = T::default();
                }
            }
            Op::Output => {
                self.output.push(self.tape[self.ptr].to_u8());
            }
            Op::Clear => {
                self.tape[self.ptr] = T::default();
            }
            Op::Set(value) => {
                self.tape[self.ptr] = match std::mem::size_of::<T>() {
                    1 => T::from_u8((*value & 0xFF) as u8),
                    2 => unsafe { std::mem::transmute_copy(&((*value & 0xFFFF) as u16)) },
                    4 => unsafe { std::mem::transmute_copy(&(*value as u32)) },
                    _ => unreachable!()
                };
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
                    
                    let result = match std::mem::size_of::<T>() {
                        1 => val.to_u8() as i32 * factor,
                        2 => {
                            let v = unsafe { std::mem::transmute_copy::<T, u16>(&val) };
                            v as i32 * factor
                        },
                        4 => {
                            let v = unsafe { std::mem::transmute_copy::<T, u32>(&val) };
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
                            break;
                        } else {
                            self.ptr = new_ptr as usize;
                        }
                    }
                    if self.ptr == 0 && !self.wrap_tape {
                        break;
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
                            break;
                        } else {
                            self.ptr = new_ptr as usize;
                        }
                    }
                }
            }
        }
    }
}

impl<'a, T: Cell> InterpreterWithCallback<'a, T> {
    pub fn new(tape_size: usize, wrap: bool, wrap_tape: bool, output_callback: &'a js_sys::Function) -> Self {
        Self {
            tape: vec![T::default(); tape_size],
            ptr: 0,
            wrap,
            wrap_tape,
            output: Vec::new(),
            input: Vec::new(),
            input_ptr: 0,
            output_callback,
        }
    }

    pub fn set_input(&mut self, input: Vec<u8>) {
        self.input = input;
        self.input_ptr = 0;
    }

    pub fn run(&mut self, ops: &[Op]) {
        for op in ops {
            self.execute(op);
        }
    }

    pub fn execute(&mut self, op: &Op) {
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
                    if new_ptr < 0 {
                        self.ptr = ((new_ptr % self.tape.len() as i64 + self.tape.len() as i64) % self.tape.len() as i64) as usize;
                    } else {
                        self.ptr = (new_ptr as usize) % self.tape.len();
                    }
                } else {
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
                if self.input_ptr < self.input.len() {
                    self.tape[self.ptr] = T::from_u8(self.input[self.input_ptr]);
                    self.input_ptr += 1;
                } else {
                    self.tape[self.ptr] = T::default();
                }
            }
            Op::Output => {
                let byte = self.tape[self.ptr].to_u8();
                self.output.push(byte);
                
                // Call the JavaScript callback with the character
                let this = JsValue::null();
                let char_str = JsValue::from_str(&(byte as char).to_string());
                let char_code = JsValue::from_f64(byte as f64);
                let _ = self.output_callback.call2(&this, &char_str, &char_code);
            }
            Op::Clear => {
                self.tape[self.ptr] = T::default();
            }
            Op::Set(value) => {
                self.tape[self.ptr] = match std::mem::size_of::<T>() {
                    1 => T::from_u8((*value & 0xFF) as u8),
                    2 => unsafe { std::mem::transmute_copy(&((*value & 0xFFFF) as u16)) },
                    4 => unsafe { std::mem::transmute_copy(&(*value as u32)) },
                    _ => unreachable!()
                };
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
                    
                    let result = match std::mem::size_of::<T>() {
                        1 => val.to_u8() as i32 * factor,
                        2 => {
                            let v = unsafe { std::mem::transmute_copy::<T, u16>(&val) };
                            v as i32 * factor
                        },
                        4 => {
                            let v = unsafe { std::mem::transmute_copy::<T, u32>(&val) };
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
                            break;
                        } else {
                            self.ptr = new_ptr as usize;
                        }
                    }
                    if self.ptr == 0 && !self.wrap_tape {
                        break;
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
                            break;
                        } else {
                            self.ptr = new_ptr as usize;
                        }
                    }
                }
            }
        }
    }
}

pub fn preprocess_collapse(code: &str) -> String {
    let mut result = String::new();
    let mut chars = code.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '+' | '-' => {
                let mut count = if ch == '+' { 1i32 } else { -1i32 };
                while let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        '+' => {
                            chars.next();
                            count += 1;
                        }
                        '-' => {
                            chars.next();
                            count -= 1;
                        }
                        _ => break,
                    }
                }
                if count > 0 {
                    for _ in 0..count {
                        result.push('+');
                    }
                } else if count < 0 {
                    for _ in 0..(-count) {
                        result.push('-');
                    }
                }
            }
            '>' | '<' => {
                let mut count = if ch == '>' { 1i32 } else { -1i32 };
                while let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        '>' => {
                            chars.next();
                            count += 1;
                        }
                        '<' => {
                            chars.next();
                            count -= 1;
                        }
                        _ => break,
                    }
                }
                if count > 0 {
                    for _ in 0..count {
                        result.push('>');
                    }
                } else if count < 0 {
                    for _ in 0..(-count) {
                        result.push('<');
                    }
                }
            }
            c if "[].,".contains(c) => result.push(c),
            _ => {}
        }
    }
    
    result
}

pub fn parse(code: &str) -> Vec<Op> {
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

pub fn optimize(ops: Vec<Op>) -> Vec<Op> {
    optimize_patterns(ops)
}

pub fn optimize_patterns(ops: Vec<Op>) -> Vec<Op> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < ops.len() {
        match &ops[i] {
            Op::Clear => {
                if i + 1 < ops.len() {
                    if let Op::Add(n) = &ops[i + 1] {
                        result.push(Op::Set(*n));
                        i += 2;
                        continue;
                    }
                }
                result.push(ops[i].clone());
                i += 1;
            }
            Op::Loop(loop_ops) => {
                let optimized_loop = optimize_patterns(loop_ops.clone());
                
                if optimized_loop.len() == 1 {
                    if let Op::Add(n) = optimized_loop[0] {
                        if n == -1 || n == 1 {
                            result.push(Op::Clear);
                            i += 1;
                            continue;
                        }
                    }
                }
                
                if optimized_loop.len() == 1 {
                    if let Op::Move(n) = optimized_loop[0] {
                        if n > 0 {
                            result.push(Op::ScanRight(n));
                            i += 1;
                            continue;
                        } else if n < 0 {
                            result.push(Op::ScanLeft(n));
                            i += 1;
                            continue;
                        }
                    }
                }
                
                if optimized_loop.len() == 4 {
                    if let (Op::Add(-1), Op::Move(m1), Op::Add(factor), Op::Move(m2)) = 
                        (&optimized_loop[0], &optimized_loop[1], &optimized_loop[2], &optimized_loop[3]) {
                        if m1 + m2 == 0 && *m1 != 0 {
                            result.push(Op::MulAdd(*m1, *factor));
                            i += 1;
                            continue;
                        }
                    }
                }
                
                result.push(Op::Loop(optimized_loop));
                i += 1;
            }
            _ => {
                result.push(ops[i].clone());
                i += 1;
            }
        }
    }
    
    result
}

#[derive(Serialize, Deserialize)]
pub struct RunResult {
    pub tape: Vec<u32>,
    pub pointer: usize,
    pub output: String,
    pub tape_truncated: bool,
    pub original_tape_size: usize,
}

#[wasm_bindgen]
pub struct BrainfuckInterpreter {
    tape_size: usize,
    cell_size: u8,
    wrap: bool,
    wrap_tape: bool,
    optimize_code: bool,
}

// Stateful interpreter that can pause and resume
#[wasm_bindgen]
pub struct StatefulBrainfuckInterpreter {
    tape: Vec<u32>,  // Store as u32 for JS compatibility
    ptr: usize,
    ops: Vec<Op>,
    pc: usize,  // Program counter
    call_stack: Vec<usize>,  // For nested loops
    loop_state: Option<LoopState>,  // Track state when paused in a loop
    output: String,
    cell_size: u8,
    wrap: bool,
    wrap_tape: bool,
    is_waiting_for_input: bool,
    is_finished: bool,
}

struct LoopState {
    ops: Vec<Op>,
    position: usize,
}

#[wasm_bindgen]
impl BrainfuckInterpreter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            tape_size: 30000,
            cell_size: 8,
            wrap: true,
            wrap_tape: true,
            optimize_code: true,
        }
    }

    #[wasm_bindgen]
    pub fn with_options(tape_size: usize, cell_size: u8, wrap: bool, wrap_tape: bool, optimize: bool) -> Self {
        Self {
            tape_size,
            cell_size,
            wrap,
            wrap_tape,
            optimize_code: optimize,
        }
    }

    #[wasm_bindgen]
    pub fn run_program(&self, code: &str, input: &[u8]) -> JsValue {
        let preprocessed = preprocess_collapse(code);
        let ops = parse(&preprocessed);
        let optimized = if self.optimize_code {
            optimize(ops)
        } else {
            ops
        };
        
        const MAX_TAPE_SIZE: usize = 10000;  // Maximum tape size to send back to JS

        match self.cell_size {
            8 => {
                let mut interpreter = Interpreter::<u8>::new(self.tape_size, self.wrap, self.wrap_tape);
                interpreter.set_input(input.to_vec());
                interpreter.run(&optimized);
                
                let (tape, truncated, original_size) = truncate_tape_u8(&interpreter.tape, interpreter.ptr, MAX_TAPE_SIZE);
                
                let result = RunResult {
                    tape,
                    pointer: interpreter.ptr,
                    output: String::from_utf8_lossy(&interpreter.output).to_string(),
                    tape_truncated: truncated,
                    original_tape_size: original_size,
                };
                
                serde_wasm_bindgen::to_value(&result).unwrap()
            }
            16 => {
                let mut interpreter = Interpreter::<u16>::new(self.tape_size, self.wrap, self.wrap_tape);
                interpreter.set_input(input.to_vec());
                interpreter.run(&optimized);
                
                let (tape, truncated, original_size) = truncate_tape_u16(&interpreter.tape, interpreter.ptr, MAX_TAPE_SIZE);
                
                let result = RunResult {
                    tape,
                    pointer: interpreter.ptr,
                    output: String::from_utf8_lossy(&interpreter.output).to_string(),
                    tape_truncated: truncated,
                    original_tape_size: original_size,
                };
                
                serde_wasm_bindgen::to_value(&result).unwrap()
            }
            32 => {
                let mut interpreter = Interpreter::<u32>::new(self.tape_size, self.wrap, self.wrap_tape);
                interpreter.set_input(input.to_vec());
                interpreter.run(&optimized);
                
                let (tape, truncated, original_size) = truncate_tape_u32(&interpreter.tape, interpreter.ptr, MAX_TAPE_SIZE);
                
                let result = RunResult {
                    tape,
                    pointer: interpreter.ptr,
                    output: String::from_utf8_lossy(&interpreter.output).to_string(),
                    tape_truncated: truncated,
                    original_tape_size: original_size,
                };
                
                serde_wasm_bindgen::to_value(&result).unwrap()
            }
            _ => panic!("Invalid cell size")
        }
    }

    #[wasm_bindgen]
    pub fn run_program_with_callback(&self, code: &str, input: &[u8], output_callback: &js_sys::Function) -> JsValue {
        let preprocessed = preprocess_collapse(code);
        let ops = parse(&preprocessed);
        let optimized = if self.optimize_code {
            optimize(ops)
        } else {
            ops
        };
        
        const MAX_TAPE_SIZE: usize = 10000;  // Maximum tape size to send back to JS

        match self.cell_size {
            8 => {
                let mut interpreter = InterpreterWithCallback::<u8>::new(
                    self.tape_size, 
                    self.wrap, 
                    self.wrap_tape,
                    output_callback
                );
                interpreter.set_input(input.to_vec());
                interpreter.run(&optimized);
                
                let (tape, truncated, original_size) = truncate_tape_u8(&interpreter.tape, interpreter.ptr, MAX_TAPE_SIZE);
                
                let result = RunResult {
                    tape,
                    pointer: interpreter.ptr,
                    output: String::from_utf8_lossy(&interpreter.output).to_string(),
                    tape_truncated: truncated,
                    original_tape_size: original_size,
                };
                
                serde_wasm_bindgen::to_value(&result).unwrap()
            }
            16 => {
                let mut interpreter = InterpreterWithCallback::<u16>::new(
                    self.tape_size, 
                    self.wrap, 
                    self.wrap_tape,
                    output_callback
                );
                interpreter.set_input(input.to_vec());
                interpreter.run(&optimized);
                
                let (tape, truncated, original_size) = truncate_tape_u16(&interpreter.tape, interpreter.ptr, MAX_TAPE_SIZE);
                
                let result = RunResult {
                    tape,
                    pointer: interpreter.ptr,
                    output: String::from_utf8_lossy(&interpreter.output).to_string(),
                    tape_truncated: truncated,
                    original_tape_size: original_size,
                };
                
                serde_wasm_bindgen::to_value(&result).unwrap()
            }
            32 => {
                let mut interpreter = InterpreterWithCallback::<u32>::new(
                    self.tape_size, 
                    self.wrap, 
                    self.wrap_tape,
                    output_callback
                );
                interpreter.set_input(input.to_vec());
                interpreter.run(&optimized);
                
                let (tape, truncated, original_size) = truncate_tape_u32(&interpreter.tape, interpreter.ptr, MAX_TAPE_SIZE);
                
                let result = RunResult {
                    tape,
                    pointer: interpreter.ptr,
                    output: String::from_utf8_lossy(&interpreter.output).to_string(),
                    tape_truncated: truncated,
                    original_tape_size: original_size,
                };
                
                serde_wasm_bindgen::to_value(&result).unwrap()
            }
            _ => panic!("Invalid cell size")
        }
    }

    #[wasm_bindgen]
    pub fn optimize_brainfuck(&self, code: &str) -> String {
        let preprocessed = preprocess_collapse(code);
        let ops = parse(&preprocessed);
        let optimized = optimize(ops);
        ops_to_brainfuck(&optimized)
    }
}

fn truncate_tape_u8(tape: &[u8], pointer: usize, max_size: usize) -> (Vec<u32>, bool, usize) {
    let tape_len = tape.len();
    
    if tape_len <= max_size {
        // Return the entire tape if it's small enough
        return (tape.iter().map(|&v| v as u32).collect(), false, tape_len);
    }
    
    // For large tapes, return a window around the pointer
    let window_size = max_size / 2;
    let start = pointer.saturating_sub(window_size);
    let end = (pointer + window_size).min(tape_len);
    
    let truncated_tape: Vec<u32> = tape[start..end]
        .iter()
        .map(|&v| v as u32)
        .collect();
    
    (truncated_tape, true, tape_len)
}

fn truncate_tape_u16(tape: &[u16], pointer: usize, max_size: usize) -> (Vec<u32>, bool, usize) {
    let tape_len = tape.len();
    
    if tape_len <= max_size {
        // Return the entire tape if it's small enough
        return (tape.iter().map(|&v| v as u32).collect(), false, tape_len);
    }
    
    // For large tapes, return a window around the pointer
    let window_size = max_size / 2;
    let start = pointer.saturating_sub(window_size);
    let end = (pointer + window_size).min(tape_len);
    
    let truncated_tape: Vec<u32> = tape[start..end]
        .iter()
        .map(|&v| v as u32)
        .collect();
    
    (truncated_tape, true, tape_len)
}

fn truncate_tape_u32(tape: &[u32], pointer: usize, max_size: usize) -> (Vec<u32>, bool, usize) {
    let tape_len = tape.len();
    
    if tape_len <= max_size {
        // Return the entire tape if it's small enough
        return (tape.clone().to_vec(), false, tape_len);
    }
    
    // For large tapes, return a window around the pointer
    let window_size = max_size / 2;
    let start = pointer.saturating_sub(window_size);
    let end = (pointer + window_size).min(tape_len);
    
    let truncated_tape: Vec<u32> = tape[start..end].to_vec();
    
    (truncated_tape, true, tape_len)
}

#[wasm_bindgen]
impl StatefulBrainfuckInterpreter {
    #[wasm_bindgen(constructor)]
    pub fn new(code: &str, tape_size: usize, cell_size: u8, wrap: bool, wrap_tape: bool, optimize_flag: bool) -> Self {
        let preprocessed = preprocess_collapse(code);
        let ops = parse(&preprocessed);
        let optimized = if optimize_flag {
            optimize(ops)
        } else {
            ops
        };
        
        // Flatten loops into a linear instruction stream with jump instructions
        let flat_ops = Self::flatten_ops(optimized);
        
        Self {
            tape: vec![0; tape_size],
            ptr: 0,
            ops: flat_ops,
            pc: 0,
            call_stack: Vec::new(),
            loop_state: None,
            output: String::new(),
            cell_size,
            wrap,
            wrap_tape,
            is_waiting_for_input: false,
            is_finished: false,
        }
    }
    
    fn flatten_ops(ops: Vec<Op>) -> Vec<Op> {
        // For simplicity, keep the nested structure for now
        // A more sophisticated implementation would flatten loops into jumps
        ops
    }
    
    #[wasm_bindgen]
    pub fn run_until_input(&mut self, output_callback: &js_sys::Function) -> bool {
        if self.is_finished {
            return false;
        }
        
        self.is_waiting_for_input = false;
        
        while self.pc < self.ops.len() {
            match &self.ops[self.pc].clone() {
                Op::Add(n) => {
                    let val = self.tape[self.ptr];
                    self.tape[self.ptr] = match self.cell_size {
                        8 => ((val as i32 + n) & 0xFF) as u32,
                        16 => ((val as i32 + n) & 0xFFFF) as u32,
                        32 => (val as i64 + *n as i64) as u32,
                        _ => val
                    };
                }
                Op::Move(n) => {
                    let new_ptr = (self.ptr as i64) + (*n as i64);
                    if self.wrap_tape {
                        if new_ptr < 0 {
                            self.ptr = ((new_ptr % self.tape.len() as i64 + self.tape.len() as i64) % self.tape.len() as i64) as usize;
                        } else {
                            self.ptr = (new_ptr as usize) % self.tape.len();
                        }
                    } else {
                        if new_ptr < 0 {
                            self.ptr = 0;
                        } else if new_ptr >= self.tape.len() as i64 {
                            self.ptr = self.tape.len() - 1;
                        } else {
                            self.ptr = new_ptr as usize;
                        }
                    }
                }
                Op::Loop(loop_ops) => {
                    if self.tape[self.ptr] == 0 {
                        // Skip the loop
                        self.pc += 1;
                    } else {
                        // Execute loop body
                        self.call_stack.push(self.pc);
                        // We need a different approach for nested ops
                        // For now, execute inline
                        self.execute_loop_properly(loop_ops, output_callback);
                        if self.is_waiting_for_input {
                            return true; // Paused for input
                        }
                    }
                    self.pc += 1;
                    continue;
                }
                Op::Input => {
                    // Signal that we need input
                    self.is_waiting_for_input = true;
                    return true; // Return true = needs input
                }
                Op::Output => {
                    let byte = (self.tape[self.ptr] & 0xFF) as u8;
                    self.output.push(byte as char);
                    
                    // Call JS callback
                    let this = JsValue::null();
                    let char_str = JsValue::from_str(&(byte as char).to_string());
                    let char_code = JsValue::from_f64(byte as f64);
                    let _ = output_callback.call2(&this, &char_str, &char_code);
                }
                Op::Clear => {
                    self.tape[self.ptr] = 0;
                }
                Op::Set(value) => {
                    self.tape[self.ptr] = match self.cell_size {
                        8 => (*value & 0xFF) as u32,
                        16 => (*value & 0xFFFF) as u32,
                        32 => *value as u32,
                        _ => 0
                    };
                }
                _ => {} // Handle other optimized operations
            }
            
            self.pc += 1;
        }
        
        self.is_finished = true;
        false // No input needed, program finished
    }
    
    fn execute_loop_properly(&mut self, ops: &[Op], output_callback: &js_sys::Function) {
        while self.tape[self.ptr] != 0 {
            // If we have saved loop state, continue from that position
            let start_pos = if let Some(ref state) = self.loop_state {
                let pos = state.position;
                self.loop_state = None;  // Clear it
                pos
            } else {
                0
            };
            
            for i in start_pos..ops.len() {
                let op = &ops[i];
                match op {
                    Op::Add(n) => {
                        let val = self.tape[self.ptr];
                        self.tape[self.ptr] = match self.cell_size {
                            8 => ((val as i32 + n) & 0xFF) as u32,
                            16 => ((val as i32 + n) & 0xFFFF) as u32,
                            32 => (val as i64 + *n as i64) as u32,
                            _ => val
                        };
                    }
                    Op::Move(n) => {
                        let new_ptr = (self.ptr as i64) + (*n as i64);
                        if self.wrap_tape {
                            if new_ptr < 0 {
                                self.ptr = ((new_ptr % self.tape.len() as i64 + self.tape.len() as i64) % self.tape.len() as i64) as usize;
                            } else {
                                self.ptr = (new_ptr as usize) % self.tape.len();
                            }
                        } else {
                            if new_ptr < 0 {
                                self.ptr = 0;
                            } else if new_ptr >= self.tape.len() as i64 {
                                self.ptr = self.tape.len() - 1;
                            } else {
                                self.ptr = new_ptr as usize;
                            }
                        }
                    }
                    Op::Loop(inner_ops) => {
                        self.execute_loop_properly(inner_ops, output_callback);
                        if self.is_waiting_for_input {
                            // Save our position in the outer loop before returning
                            self.loop_state = Some(LoopState {
                                ops: ops.to_vec(),
                                position: i + 1,  // Resume from next instruction
                            });
                            return;
                        }
                    }
                    Op::Input => {
                        self.is_waiting_for_input = true;
                        // Save state: we need to resume from the next instruction
                        self.loop_state = Some(LoopState {
                            ops: ops.to_vec(),
                            position: i + 1,  // Resume from next instruction
                        });
                        return;
                    }
                    Op::Output => {
                        let byte = (self.tape[self.ptr] & 0xFF) as u8;
                        self.output.push(byte as char);
                        
                        let this = JsValue::null();
                        let char_str = JsValue::from_str(&(byte as char).to_string());
                        let char_code = JsValue::from_f64(byte as f64);
                        let _ = output_callback.call2(&this, &char_str, &char_code);
                    }
                    Op::Clear => {
                        self.tape[self.ptr] = 0;
                    }
                    Op::Set(value) => {
                        self.tape[self.ptr] = match self.cell_size {
                            8 => (*value & 0xFF) as u32,
                            16 => (*value & 0xFFFF) as u32,
                            32 => *value as u32,
                            _ => 0
                        };
                    }
                    _ => {} // Handle other optimized operations
                }
            }
        }
    }
    
    #[wasm_bindgen]
    pub fn provide_input(&mut self, char_code: u8) {
        if self.is_waiting_for_input {
            self.tape[self.ptr] = char_code as u32;
            self.is_waiting_for_input = false;
            // If we're in a loop, the loop state handles resumption
            // If we're in the main program, advance pc
            if self.loop_state.is_none() {
                self.pc += 1; // Move past the input instruction
            }
        }
    }
    
    #[wasm_bindgen]
    pub fn get_state(&self) -> JsValue {
        let result = RunResult {
            tape: self.tape.clone(),
            pointer: self.ptr,
            output: self.output.clone(),
            tape_truncated: false,
            original_tape_size: self.tape.len(),
        };
        serde_wasm_bindgen::to_value(&result).unwrap()
    }
    
    #[wasm_bindgen]
    pub fn is_waiting_for_input(&self) -> bool {
        self.is_waiting_for_input
    }
    
    #[wasm_bindgen]
    pub fn is_finished(&self) -> bool {
        self.is_finished
    }
}

fn ops_to_brainfuck(ops: &[Op]) -> String {
    let mut result = String::new();
    
    for op in ops {
        match op {
            Op::Add(n) => {
                if *n > 0 {
                    for _ in 0..*n {
                        result.push('+');
                    }
                } else if *n < 0 {
                    for _ in 0..(-n) {
                        result.push('-');
                    }
                }
            }
            Op::Move(n) => {
                if *n > 0 {
                    for _ in 0..*n {
                        result.push('>');
                    }
                } else if *n < 0 {
                    for _ in 0..(-n) {
                        result.push('<');
                    }
                }
            }
            Op::Loop(loop_ops) => {
                result.push('[');
                result.push_str(&ops_to_brainfuck(loop_ops));
                result.push(']');
            }
            Op::Input => result.push(','),
            Op::Output => result.push('.'),
            Op::Clear => result.push_str("[-]"),
            Op::Set(n) => {
                result.push_str("[-]");
                if *n > 0 {
                    for _ in 0..*n {
                        result.push('+');
                    }
                } else if *n < 0 {
                    for _ in 0..(-n) {
                        result.push('-');
                    }
                }
            }
            Op::MulAdd(offset, factor) => {
                result.push('[');
                result.push('-');
                if *offset > 0 {
                    for _ in 0..*offset {
                        result.push('>');
                    }
                } else {
                    for _ in 0..(-offset) {
                        result.push('<');
                    }
                }
                if *factor > 0 {
                    for _ in 0..*factor {
                        result.push('+');
                    }
                } else {
                    for _ in 0..(-factor) {
                        result.push('-');
                    }
                }
                if *offset > 0 {
                    for _ in 0..*offset {
                        result.push('<');
                    }
                } else {
                    for _ in 0..(-offset) {
                        result.push('>');
                    }
                }
                result.push(']');
            }
            Op::ScanLeft(step) => {
                result.push('[');
                for _ in 0..step.abs() {
                    result.push('<');
                }
                result.push(']');
            }
            Op::ScanRight(step) => {
                result.push('[');
                for _ in 0..*step {
                    result.push('>');
                }
                result.push(']');
            }
        }
    }
    
    result
}