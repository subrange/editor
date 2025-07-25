use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use web_sys::console;
use web_sys::js_sys::Date;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellSize {
    Bit8,
    Bit16,
    Bit32,
}

impl CellSize {
    fn from_u32(value: u32) -> Option<Self> {
        match value {
            256 => Some(CellSize::Bit8),
            65536 => Some(CellSize::Bit16),
            _ => None,  // 32-bit cells not supported in wasm32
        }
    }
    
    fn as_u32(&self) -> u32 {
        match self {
            CellSize::Bit8 => 256,
            CellSize::Bit16 => 65536,
            CellSize::Bit32 => 65536,  // Fall back to 16-bit for now
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpreterState {
    pub pointer: usize,
    pub is_running: bool,
    pub is_paused: bool,
    pub is_stopped: bool,
    pub output: String,
    pub lane_count: usize,
}

#[wasm_bindgen]
pub struct BrainfuckInterpreter {
    tape: Vec<u32>,
    tape_size: usize,
    cell_size: CellSize,
    pointer: usize,
    code: Vec<Line>,
    current_pos: Position,
    loop_map: HashMap<String, Position>,
    breakpoints: Vec<Position>,
    last_paused_breakpoint: Option<Position>,
    output: String,
    is_running: bool,
    is_paused: bool,
    is_stopped: bool,
    lane_count: usize,
    compiled_ops: Vec<Operation>,
    jump_table: HashMap<usize, usize>,
    turbo_pc: usize,
    turbo_ops_executed: u64,
    turbo_start_time: Option<f64>,
    is_8bit: bool,
    unsafe_mode: bool,  // Skip overflow checks for maximum performance
    source_ops_count: u64,  // Count of original brainfuck operations (not optimized)
}

#[derive(Debug, Clone)]
enum Operation {
    MoveRight(usize),
    MoveLeft(usize),
    Increment(usize),
    Decrement(usize),
    LoopStart(usize),
    LoopEnd(usize),
    Output,
    Input,
    // Optimized operations
    SetZero,              // [-] pattern
    MoveValue(isize),     // [->+<] pattern
    MultiplyMove(Vec<(isize, i32)>), // Complex move patterns
    AddOffset(isize, i32), // Add value to cell at offset
    SetValue(u32),        // Set cell to specific value
    ScanRight,            // [>] pattern - scan for zero cell to the right
    ScanLeft,             // [<] pattern - scan for zero cell to the left
}

#[wasm_bindgen]
impl BrainfuckInterpreter {
    #[wasm_bindgen(constructor)]
    pub fn new(tape_size: usize, cell_size: u32) -> Result<BrainfuckInterpreter, JsValue> {
        console::log_1(&"Creating new Brainfuck interpreter in Rust".into());
        
        let cell_size = CellSize::from_u32(cell_size)
            .ok_or_else(|| JsValue::from_str("Invalid cell size"))?;
        
        let tape = vec![0u32; tape_size];
        let is_8bit = cell_size.as_u32() == 256;
        
        Ok(BrainfuckInterpreter {
            tape,
            tape_size,
            cell_size,
            pointer: 0,
            code: Vec::new(),
            current_pos: Position { line: 0, column: 0 },
            loop_map: HashMap::new(),
            breakpoints: Vec::new(),
            last_paused_breakpoint: None,
            output: String::new(),
            is_running: false,
            is_paused: false,
            is_stopped: false,
            lane_count: 1,
            compiled_ops: Vec::new(),
            jump_table: HashMap::new(),
            turbo_pc: 0,
            turbo_ops_executed: 0,
            turbo_start_time: None,
            is_8bit,
            unsafe_mode: false,
            source_ops_count: 0,
        })
    }

    #[wasm_bindgen]
    pub fn set_code(&mut self, code_json: &str) -> Result<(), JsValue> {
        let code: Vec<Line> = serde_json::from_str(code_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse code: {}", e)))?;
        
        self.code = code;
        self.build_loop_map();
        self.compile_program();
        self.reset();
        
        Ok(())
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.tape.fill(0);
        self.pointer = 0;
        self.current_pos = Position { line: 0, column: 0 };
        self.output.clear();
        self.is_running = false;
        self.is_paused = false;
        self.is_stopped = false;
        self.last_paused_breakpoint = None;
        self.turbo_pc = 0;
        self.turbo_ops_executed = 0;
        self.turbo_start_time = None;
    }

    #[wasm_bindgen]
    pub fn step(&mut self) -> bool {
        if self.is_stopped {
            return false;
        }

        let char_opt = self.get_current_char();
        
        if let Some(char) = char_opt {
            if "+-<>[].,".contains(char) && self.should_pause_at_breakpoint(&self.current_pos) {
                let is_same_breakpoint = self.last_paused_breakpoint
                    .map(|bp| bp.line == self.current_pos.line && bp.column == self.current_pos.column)
                    .unwrap_or(false);
                
                if !is_same_breakpoint {
                    console::log_1(&format!("Hit breakpoint at line {}, column {}", 
                        self.current_pos.line, self.current_pos.column).into());
                    self.last_paused_breakpoint = Some(self.current_pos);
                    self.pause();
                    return true;
                }
            }
        }

        if self.last_paused_breakpoint.is_some() {
            let bp = self.last_paused_breakpoint.unwrap();
            if bp.line != self.current_pos.line || bp.column != self.current_pos.column {
                self.last_paused_breakpoint = None;
            }
        }

        match char_opt {
            Some('/') => {
                if !self.move_to_next_line() {
                    self.stop();
                    return false;
                }
                return self.step();
            }
            Some(char) if !"+-<>[].,".contains(char) => {
                if !self.move_to_next_char() {
                    self.stop();
                    return false;
                }
                return self.step();
            }
            None => {
                if !self.move_to_next_char() {
                    self.stop();
                    return false;
                }
                return self.step();
            }
            Some(char) => {
                self.execute_instruction(char);
                
                if !self.move_to_next_char() {
                    self.stop();
                    return false;
                }
                
                true
            }
        }
    }

    #[wasm_bindgen]
    pub fn run_turbo(&mut self) -> Result<(), JsValue> {
        let start_time = Date::now();
        self.turbo_start_time = Some(start_time);
        self.is_running = true;
        self.is_paused = false;
        self.is_stopped = false;

        let mut pc = 0;
        let mut ops_executed = 0u64;
        
        // For maximum performance in unsafe mode, inline the hot operations
        if self.unsafe_mode && self.is_8bit {
            self.run_turbo_unsafe_8bit(pc, &mut ops_executed);
        } else {
            while pc < self.compiled_ops.len() && self.is_running && !self.is_paused {
                pc = self.execute_optimized_op(pc);
                ops_executed += 1;
            }
        }
        
        let end_time = Date::now();
        let total_time = (end_time - start_time) / 1000.0; // Convert to seconds
        let ops_per_sec = ops_executed as f64 / total_time;
        
        self.is_running = false;
        console::log_1(&format!("Turbo execution completed!").into());
        console::log_1(&format!("  Total operations: {}", ops_executed).into());
        console::log_1(&format!("  Total time: {:.3} seconds", total_time).into());
        console::log_1(&format!("  Performance: {:.2} million operations/second", ops_per_sec / 1_000_000.0).into());
        
        Ok(())
    }
    
    fn execute_optimized_op(&mut self, pc: usize) -> usize {
        unsafe {
            match self.compiled_ops.get_unchecked(pc) {
                Operation::MoveRight(n) => {
                    self.pointer = (self.pointer + n) % self.tape_size;
                    pc + 1
                }
                Operation::MoveLeft(n) => {
                    self.pointer = (self.pointer + self.tape_size - (n % self.tape_size)) % self.tape_size;
                    pc + 1
                }
            Operation::Increment(n) => {
                let cell = self.tape.get_unchecked_mut(self.pointer);
                if self.unsafe_mode {
                    // In unsafe mode, just add without any overflow checking
                    *cell = cell.wrapping_add(*n as u32);
                } else if self.is_8bit {
                    *cell = (*cell as u8).wrapping_add(*n as u8) as u32;
                } else {
                    let cell_max = self.cell_size.as_u32();
                    *cell = (*cell + *n as u32) % cell_max;
                }
                pc + 1
            }
            Operation::Decrement(n) => {
                let cell = self.tape.get_unchecked_mut(self.pointer);
                if self.unsafe_mode {
                    // In unsafe mode, just subtract without any overflow checking
                    *cell = cell.wrapping_sub(*n as u32);
                } else if self.is_8bit {
                    *cell = (*cell as u8).wrapping_sub(*n as u8) as u32;
                } else {
                    let cell_max = self.cell_size.as_u32();
                    let n_mod = (*n % cell_max as usize) as u32;
                    *cell = (*cell + cell_max - n_mod) % cell_max;
                }
                pc + 1
            }
            Operation::LoopStart(end_pc) => {
                if *self.tape.get_unchecked(self.pointer) == 0 {
                    *end_pc  // Jump to the closing bracket
                } else {
                    pc + 1
                }
            }
            Operation::LoopEnd(start_pc) => {
                if *self.tape.get_unchecked(self.pointer) != 0 {
                    *start_pc  // Jump TO the opening bracket
                } else {
                    pc + 1
                }
            }
            Operation::Output => {
                self.output.push(char::from_u32(*self.tape.get_unchecked(self.pointer)).unwrap_or('?'));
                pc + 1
            }
            Operation::Input => {
                console::log_1(&format!("Input requested at position {}", self.pointer).into());
                pc + 1
            }
            Operation::SetZero => {
                *self.tape.get_unchecked_mut(self.pointer) = 0;
                pc + 1
            }
            Operation::MoveValue(offset) => {
                let value = self.tape[self.pointer];
                self.tape[self.pointer] = 0;
                let target = if *offset >= 0 {
                    (self.pointer + *offset as usize) % self.tape_size
                } else {
                    (self.pointer + self.tape_size - ((-*offset) as usize % self.tape_size)) % self.tape_size
                };
                let cell_max = self.cell_size.as_u32();
                self.tape[target] = (self.tape[target] + value) % cell_max;
                pc + 1
            }
            Operation::MultiplyMove(moves) => {
                let value = self.tape[self.pointer];
                self.tape[self.pointer] = 0;
                let cell_max = self.cell_size.as_u32();
                
                for (offset, multiplier) in moves {
                    let target = if *offset >= 0 {
                        (self.pointer + *offset as usize) % self.tape_size
                    } else {
                        (self.pointer + self.tape_size - ((-*offset) as usize % self.tape_size)) % self.tape_size
                    };
                    
                    if *multiplier >= 0 {
                        let add_value = (value as u64 * *multiplier as u64) % cell_max as u64;
                        self.tape[target] = ((self.tape[target] as u64 + add_value) % cell_max as u64) as u32;
                    } else {
                        let sub_value = (value as u64 * (-*multiplier) as u64) % cell_max as u64;
                        self.tape[target] = ((self.tape[target] as u64 + cell_max as u64 - sub_value) % cell_max as u64) as u32;
                    }
                }
                pc + 1
            }
            Operation::AddOffset(offset, value) => {
                let target = if *offset >= 0 {
                    (self.pointer + *offset as usize) % self.tape_size
                } else {
                    (self.pointer + self.tape_size - ((-*offset) as usize % self.tape_size)) % self.tape_size
                };
                
                if self.unsafe_mode {
                    let cell = self.tape.get_unchecked_mut(target);
                    if *value >= 0 {
                        *cell = cell.wrapping_add(*value as u32);
                    } else {
                        *cell = cell.wrapping_sub((-*value) as u32);
                    }
                } else {
                    let cell_max = self.cell_size.as_u32();
                    if *value >= 0 {
                        self.tape[target] = (self.tape[target] + *value as u32) % cell_max;
                    } else {
                        self.tape[target] = (self.tape[target] + cell_max - (-*value) as u32) % cell_max;
                    }
                }
                pc + 1
            }
            Operation::SetValue(value) => {
                if self.unsafe_mode {
                    *self.tape.get_unchecked_mut(self.pointer) = *value;
                } else {
                    self.tape[self.pointer] = *value % self.cell_size.as_u32();
                }
                pc + 1
            }
            Operation::ScanRight => {
                while self.pointer < self.tape_size && *self.tape.get_unchecked(self.pointer) != 0 {
                    self.pointer += 1;
                }
                if self.pointer >= self.tape_size {
                    self.pointer = self.tape_size - 1;
                }
                pc + 1
            }
            Operation::ScanLeft => {
                while self.pointer > 0 && *self.tape.get_unchecked(self.pointer) != 0 {
                    self.pointer -= 1;
                }
                pc + 1
            }
            }
        }
    }

    #[wasm_bindgen]
    pub fn run_turbo_batch(&mut self, batch_size: usize) -> Result<bool, JsValue> {
        if !self.is_running {
            self.is_running = true;
            self.is_paused = false;
            self.is_stopped = false;
            self.turbo_pc = 0;
            self.turbo_ops_executed = 0;
            self.turbo_start_time = Some(Date::now());
        }

        let mut ops_in_batch = 0;
        
        while self.turbo_pc < self.compiled_ops.len() && self.is_running && !self.is_paused && ops_in_batch < batch_size {
            self.turbo_pc = self.execute_optimized_op(self.turbo_pc);
            self.turbo_ops_executed += 1;
            ops_in_batch += 1;
        }
        
        if self.turbo_pc >= self.compiled_ops.len() {
            self.is_running = false;
            
            // Log final performance stats
            if let Some(start_time) = self.turbo_start_time {
                let total_time = (Date::now() - start_time) / 1000.0;
                let ops_per_sec = self.turbo_ops_executed as f64 / total_time;
                console::log_1(&format!("=== Turbo Execution Complete ===").into());
                console::log_1(&format!("Total time: {:.3} seconds", total_time).into());
                console::log_1(&format!("Source BF operations: {}", self.source_ops_count).into());
                console::log_1(&format!("Optimized operations executed: {}", self.turbo_ops_executed).into());
                console::log_1(&format!("Optimization ratio: {:.2}x", self.source_ops_count as f64 / self.turbo_ops_executed as f64).into());
                console::log_1(&format!("Performance: {:.2} million ops/sec", ops_per_sec / 1_000_000.0).into());
                console::log_1(&format!("==============================").into());
            }
            
            Ok(false) // No more work to do
        } else {
            Ok(true) // More work remains
        }
    }
    
    #[wasm_bindgen]
    pub fn get_performance_stats(&self) -> String {
        if let Some(start_time) = self.turbo_start_time {
            let elapsed = (Date::now() - start_time) / 1000.0; // seconds
            let ops_per_sec = self.turbo_ops_executed as f64 / elapsed;
            format!("Operations: {} | Time: {:.3}s | Speed: {:.2}M ops/sec", 
                self.turbo_ops_executed, 
                elapsed,
                ops_per_sec / 1_000_000.0)
        } else {
            format!("Operations executed: {}", self.turbo_ops_executed)
        }
    }

    #[wasm_bindgen]
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    #[wasm_bindgen]
    pub fn resume(&mut self) {
        console::log_1(&format!("Resume called: is_running={}, is_paused={}", self.is_running, self.is_paused).into());
        
        // Always clear the pause flag when resume is called
        self.is_paused = false;
        console::log_1(&format!("Resumed at position {:?}", self.current_pos).into());
    }

    #[wasm_bindgen]
    pub fn stop(&mut self) {
        self.is_running = false;
        self.is_paused = false;
        self.is_stopped = true;
        self.last_paused_breakpoint = None;
    }

    #[wasm_bindgen]
    pub fn toggle_breakpoint(&mut self, line: usize, column: usize) {
        let pos = Position { line, column };
        
        if let Some(index) = self.breakpoints.iter().position(|bp| bp.line == line && bp.column == column) {
            self.breakpoints.remove(index);
        } else {
            self.breakpoints.push(pos);
        }
    }

    #[wasm_bindgen]
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
        self.last_paused_breakpoint = None;
    }

    #[wasm_bindgen]
    pub fn get_state(&self) -> Result<String, JsValue> {
        let state = InterpreterState {
            pointer: self.pointer,
            is_running: self.is_running,
            is_paused: self.is_paused,
            is_stopped: self.is_stopped,
            output: self.output.clone(),
            lane_count: self.lane_count,
        };
        
        serde_json::to_string(&state)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize state: {}", e)))
    }

    #[wasm_bindgen]
    pub fn get_tape_slice(&self, start: usize, end: usize) -> Vec<u32> {
        let end = end.min(self.tape.len());
        self.tape[start..end].to_vec()
    }

    #[wasm_bindgen]
    pub fn get_output(&self) -> String {
        self.output.clone()
    }

    #[wasm_bindgen]
    pub fn get_pointer(&self) -> usize {
        self.pointer
    }

    #[wasm_bindgen]
    pub fn get_current_position(&self) -> Result<String, JsValue> {
        let pos = Position {
            line: self.current_pos.line,
            column: self.current_pos.column,
        };
        
        serde_json::to_string(&pos)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize position: {}", e)))
    }

    #[wasm_bindgen]
    pub fn set_tape_size(&mut self, size: usize) {
        self.tape_size = size;
        self.tape = vec![0u32; size];
        self.reset();
    }

    #[wasm_bindgen]
    pub fn set_cell_size(&mut self, size: u32) -> Result<(), JsValue> {
        let cell_size = CellSize::from_u32(size)
            .ok_or_else(|| JsValue::from_str("Invalid cell size"))?;
        
        self.cell_size = cell_size;
        self.is_8bit = cell_size.as_u32() == 256;
        self.reset();
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_lane_count(&mut self, count: usize) -> Result<(), JsValue> {
        if count < 1 || count > 10 {
            return Err(JsValue::from_str("Lane count must be between 1 and 10"));
        }
        self.lane_count = count;
        Ok(())
    }
    
    #[wasm_bindgen]
    pub fn set_unsafe_mode(&mut self, enabled: bool) {
        self.unsafe_mode = enabled;
        if enabled {
            console::log_1(&"Unsafe mode enabled: No overflow checking for maximum performance".into());
        }
    }
}

impl BrainfuckInterpreter {
    // Ultra-fast execution path for 8-bit unsafe mode
    fn run_turbo_unsafe_8bit(&mut self, mut pc: usize, ops_executed: &mut u64) {
        unsafe {
            let ops = &self.compiled_ops;
            let tape = &mut self.tape;
            let tape_size = self.tape_size;
            
            while pc < ops.len() && self.is_running && !self.is_paused {
                match ops.get_unchecked(pc) {
                    Operation::MoveRight(n) => {
                        self.pointer = (self.pointer + n) % tape_size;
                    }
                    Operation::MoveLeft(n) => {
                        self.pointer = (self.pointer + tape_size - (n % tape_size)) % tape_size;
                    }
                    Operation::Increment(n) => {
                        let cell = tape.get_unchecked_mut(self.pointer);
                        *cell = (*cell).wrapping_add(*n as u32);
                    }
                    Operation::Decrement(n) => {
                        let cell = tape.get_unchecked_mut(self.pointer);
                        *cell = (*cell).wrapping_sub(*n as u32);
                    }
                    Operation::LoopStart(end_pc) => {
                        if *tape.get_unchecked(self.pointer) == 0 {
                            pc = *end_pc;
                        }
                    }
                    Operation::LoopEnd(start_pc) => {
                        if *tape.get_unchecked(self.pointer) != 0 {
                            pc = *start_pc - 1; // -1 because we'll increment at the end
                        }
                    }
                    Operation::SetZero => {
                        *tape.get_unchecked_mut(self.pointer) = 0;
                    }
                    Operation::MoveValue(offset) => {
                        let value = *tape.get_unchecked(self.pointer);
                        *tape.get_unchecked_mut(self.pointer) = 0;
                        let target = if *offset >= 0 {
                            (self.pointer + *offset as usize) % tape_size
                        } else {
                            (self.pointer + tape_size - ((-*offset) as usize % tape_size)) % tape_size
                        };
                        let target_cell = tape.get_unchecked_mut(target);
                        *target_cell = (*target_cell).wrapping_add(value);
                    }
                    Operation::MultiplyMove(moves) => {
                        let value = *tape.get_unchecked(self.pointer);
                        *tape.get_unchecked_mut(self.pointer) = 0;
                        
                        for (offset, multiplier) in moves {
                            let target = if *offset >= 0 {
                                (self.pointer + *offset as usize) % tape_size
                            } else {
                                (self.pointer + tape_size - ((-*offset) as usize % tape_size)) % tape_size
                            };
                            let target_cell = tape.get_unchecked_mut(target);
                            if *multiplier >= 0 {
                                *target_cell = (*target_cell).wrapping_add((value as i64 * *multiplier as i64) as u32);
                            } else {
                                *target_cell = (*target_cell).wrapping_sub((value as i64 * (-*multiplier) as i64) as u32);
                            }
                        }
                    }
                    Operation::AddOffset(offset, value) => {
                        let target = if *offset >= 0 {
                            (self.pointer + *offset as usize) % tape_size
                        } else {
                            (self.pointer + tape_size - ((-*offset) as usize % tape_size)) % tape_size
                        };
                        let cell = tape.get_unchecked_mut(target);
                        if *value >= 0 {
                            *cell = (*cell).wrapping_add(*value as u32);
                        } else {
                            *cell = (*cell).wrapping_sub((-*value) as u32);
                        }
                    }
                    Operation::SetValue(value) => {
                        *tape.get_unchecked_mut(self.pointer) = *value;
                    }
                    Operation::ScanRight => {
                        while self.pointer < tape_size && *tape.get_unchecked(self.pointer) != 0 {
                            self.pointer += 1;
                        }
                        if self.pointer >= tape_size {
                            self.pointer = tape_size - 1;
                        }
                    }
                    Operation::ScanLeft => {
                        while self.pointer > 0 && *tape.get_unchecked(self.pointer) != 0 {
                            self.pointer -= 1;
                        }
                    }
                    Operation::Output => {
                        self.output.push(char::from_u32(*tape.get_unchecked(self.pointer)).unwrap_or('?'));
                    }
                    Operation::Input => {
                        // Input not supported in turbo mode
                    }
                }
                
                pc += 1;
                *ops_executed += 1;
            }
        }
    }
    
    fn build_loop_map(&mut self) {
        self.loop_map.clear();
        let mut stack = Vec::new();

        for (line_idx, line) in self.code.iter().enumerate() {
            for (col_idx, ch) in line.text.chars().enumerate() {
                let pos = Position { line: line_idx, column: col_idx };
                
                match ch {
                    '[' => stack.push(pos),
                    ']' => {
                        if let Some(open_pos) = stack.pop() {
                            self.loop_map.insert(Self::pos_to_key(&open_pos), pos);
                            self.loop_map.insert(Self::pos_to_key(&pos), open_pos);
                        } else {
                            console::log_1(&format!("Unmatched ] at line {}, column {}", line_idx, col_idx).into());
                        }
                    }
                    _ => {}
                }
            }
        }

        if !stack.is_empty() {
            console::log_1(&format!("Unmatched [ brackets: {} remaining", stack.len()).into());
        }
    }

    fn compile_program(&mut self) {
        self.compiled_ops.clear();
        self.jump_table.clear();
        self.source_ops_count = 0;
        
        // First pass: convert to basic operations
        let mut basic_ops = Vec::new();
        let mut jump_stack = Vec::new();
        
        for line in &self.code {
            for ch in line.text.chars() {
                let op_index = basic_ops.len();
                
                match ch {
                    '>' => {
                        basic_ops.push(Operation::MoveRight(1));
                        self.source_ops_count += 1;
                    },
                    '<' => {
                        basic_ops.push(Operation::MoveLeft(1));
                        self.source_ops_count += 1;
                    },
                    '+' => {
                        basic_ops.push(Operation::Increment(1));
                        self.source_ops_count += 1;
                    },
                    '-' => {
                        basic_ops.push(Operation::Decrement(1));
                        self.source_ops_count += 1;
                    },
                    '[' => {
                        jump_stack.push(op_index);
                        basic_ops.push(Operation::LoopStart(0));
                        self.source_ops_count += 1;
                    }
                    ']' => {
                        if let Some(start_index) = jump_stack.pop() {
                            basic_ops[start_index] = Operation::LoopStart(op_index);
                            basic_ops.push(Operation::LoopEnd(start_index));
                        }
                        self.source_ops_count += 1;
                    }
                    '.' => {
                        basic_ops.push(Operation::Output);
                        self.source_ops_count += 1;
                    },
                    ',' => {
                        basic_ops.push(Operation::Input);
                        self.source_ops_count += 1;
                    },
                    _ => {}
                }
            }
        }
        
        // Second pass: optimize
        self.compiled_ops = self.optimize_operations(basic_ops);
        
        // Third pass: fix jump addresses after optimization
        self.fix_jump_addresses();
    }
    
    fn optimize_operations(&self, ops: Vec<Operation>) -> Vec<Operation> {
        let mut optimized = Vec::new();
        let mut i = 0;
        
        // First pass: basic optimizations
        while i < ops.len() {
            match &ops[i] {
                Operation::LoopStart(end) => {
                    // Check for [-] pattern (clear loop)
                    if i + 2 < ops.len() && *end == i + 2 {
                        if matches!(ops[i + 1], Operation::Decrement(1)) {
                            optimized.push(Operation::SetZero);
                            i += 3; // Skip the entire loop
                            continue;
                        }
                    }
                    
                    // Check for scan patterns [>] or [<]
                    if i + 2 < ops.len() && *end == i + 2 {
                        if matches!(ops[i + 1], Operation::MoveRight(1)) {
                            optimized.push(Operation::ScanRight);
                            i += 3;
                            continue;
                        }
                        if matches!(ops[i + 1], Operation::MoveLeft(1)) {
                            optimized.push(Operation::ScanLeft);
                            i += 3;
                            continue;
                        }
                    }
                    
                    // Check for simple move patterns [->+<]
                    if let Some(pattern) = self.detect_move_pattern(&ops, i, *end) {
                        optimized.push(pattern);
                        i = *end + 1;
                        continue;
                    }
                    
                    optimized.push(ops[i].clone());
                    i += 1;
                }
                
                // Batch consecutive operations
                Operation::MoveRight(_) => {
                    let mut count = 0;
                    while i < ops.len() {
                        match &ops[i] {
                            Operation::MoveRight(n) => count += n,
                            _ => break,
                        }
                        i += 1;
                    }
                    if count > 0 {
                        optimized.push(Operation::MoveRight(count));
                    }
                }
                
                Operation::MoveLeft(_) => {
                    let mut count = 0;
                    while i < ops.len() {
                        match &ops[i] {
                            Operation::MoveLeft(n) => count += n,
                            _ => break,
                        }
                        i += 1;
                    }
                    if count > 0 {
                        optimized.push(Operation::MoveLeft(count));
                    }
                }
                
                Operation::Increment(_) => {
                    let mut count = 0;
                    while i < ops.len() {
                        match &ops[i] {
                            Operation::Increment(n) => count += n,
                            _ => break,
                        }
                        i += 1;
                    }
                    if count > 0 {
                        optimized.push(Operation::Increment(count));
                    }
                }
                
                Operation::Decrement(_) => {
                    let mut count = 0;
                    while i < ops.len() {
                        match &ops[i] {
                            Operation::Decrement(n) => count += n,
                            _ => break,
                        }
                        i += 1;
                    }
                    if count > 0 {
                        optimized.push(Operation::Decrement(count));
                    }
                }
                
                _ => {
                    optimized.push(ops[i].clone());
                    i += 1;
                }
            }
        }
        
        // Second pass: peephole optimizations
        self.peephole_optimize(optimized)
    }
    
    fn peephole_optimize(&self, ops: Vec<Operation>) -> Vec<Operation> {
        let mut i = 0;
        let mut optimized = Vec::new();
        
        while i < ops.len() {
            match &ops[i] {
                // Combine SetZero followed by Increment/Decrement into SetValue
                Operation::SetZero => {
                    if i + 1 < ops.len() {
                        match &ops[i + 1] {
                            Operation::Increment(n) => {
                                optimized.push(Operation::SetValue(*n as u32));
                                i += 2;
                                continue;
                            }
                            _ => {}
                        }
                    }
                    optimized.push(ops[i].clone());
                    i += 1;
                }
                // Optimize pointer movements followed by operations
                Operation::MoveRight(n) => {
                    let mut combined_offset = *n as isize;
                    let mut j = i + 1;
                    let mut add_ops = Vec::new();
                    
                    // Look ahead for operations that can be combined
                    while j < ops.len() {
                        match &ops[j] {
                            Operation::Increment(inc) => {
                                add_ops.push((combined_offset, *inc as i32));
                                j += 1;
                            }
                            Operation::Decrement(dec) => {
                                add_ops.push((combined_offset, -(*dec as i32)));
                                j += 1;
                            }
                            Operation::MoveRight(n2) => {
                                combined_offset += *n2 as isize;
                                j += 1;
                            }
                            Operation::MoveLeft(n2) => {
                                combined_offset -= *n2 as isize;
                                j += 1;
                            }
                            _ => break,
                        }
                    }
                    
                    // Emit optimized operations
                    if !add_ops.is_empty() && combined_offset == 0 {
                        // All operations at offsets, pointer returns to original position
                        for (offset, value) in add_ops {
                            optimized.push(Operation::AddOffset(offset, value));
                        }
                        i = j;
                    } else {
                        optimized.push(ops[i].clone());
                        i += 1;
                    }
                }
                _ => {
                    optimized.push(ops[i].clone());
                    i += 1;
                }
            }
        }
        
        optimized
    }
    
    fn fix_jump_addresses(&mut self) {
        let mut jump_stack = Vec::new();
        
        for i in 0..self.compiled_ops.len() {
            match &mut self.compiled_ops[i] {
                Operation::LoopStart(_) => {
                    jump_stack.push(i);
                }
                Operation::LoopEnd(_) => {
                    if let Some(start_idx) = jump_stack.pop() {
                        // Update both jump addresses to point to each other
                        self.compiled_ops[start_idx] = Operation::LoopStart(i);
                        self.compiled_ops[i] = Operation::LoopEnd(start_idx);
                    }
                }
                _ => {}
            }
        }
    }
    
    fn detect_move_pattern(&self, ops: &[Operation], start: usize, end: usize) -> Option<Operation> {
        if end <= start + 2 {
            return None;
        }
        
        let loop_ops = &ops[start + 1..end];
        
        // Check if it's a simple move pattern
        let mut movements = std::collections::HashMap::new();
        let mut current_offset = 0isize;
        
        for op in loop_ops {
            match op {
                Operation::MoveRight(n) => current_offset += *n as isize,
                Operation::MoveLeft(n) => current_offset -= *n as isize,
                Operation::Increment(n) => {
                    *movements.entry(current_offset).or_insert(0) += *n as i32;
                }
                Operation::Decrement(n) => {
                    *movements.entry(current_offset).or_insert(0) -= *n as i32;
                }
                _ => return None, // Complex operation, bail out
            }
        }
        
        // Must end at starting position
        if current_offset != 0 {
            return None;
        }
        
        // Check if it's a valid pattern - current cell must change by exactly -1 per iteration
        let current_cell_delta = movements.get(&0).copied().unwrap_or(0);
        if current_cell_delta != -1 && current_cell_delta != 1 {
            return None;
        }
        
        // If current cell increments, we need to negate all the movements
        let multiplier = if current_cell_delta == 1 { -1 } else { 1 };
        
        movements.remove(&0);
        
        // If only one other cell is modified by +1, it's a simple move
        if movements.len() == 1 {
            if let Some((&offset, &delta)) = movements.iter().next() {
                if delta == 1 * multiplier {
                    return Some(Operation::MoveValue(offset));
                }
            }
        }
        
        // Otherwise, it's a multiply-move pattern
        if !movements.is_empty() {
            let mut moves: Vec<(isize, i32)> = movements.into_iter()
                .map(|(offset, delta)| (offset, delta * multiplier))
                .collect();
            moves.sort_by_key(|&(offset, _)| offset);
            return Some(Operation::MultiplyMove(moves));
        }
        
        None
    }

    fn pos_to_key(pos: &Position) -> String {
        format!("{},{}", pos.line, pos.column)
    }

    fn get_current_char(&self) -> Option<char> {
        self.code.get(self.current_pos.line)
            .and_then(|line| line.text.chars().nth(self.current_pos.column))
    }

    fn move_to_next_char(&mut self) -> bool {
        if let Some(line) = self.code.get(self.current_pos.line) {
            if self.current_pos.column < line.text.len() - 1 {
                self.current_pos.column += 1;
                true
            } else if self.current_pos.line < self.code.len() - 1 {
                self.current_pos.line += 1;
                self.current_pos.column = 0;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn move_to_next_line(&mut self) -> bool {
        if self.current_pos.line < self.code.len() - 1 {
            self.current_pos.line += 1;
            self.current_pos.column = 0;
            true
        } else {
            false
        }
    }

    fn execute_instruction(&mut self, instruction: char) {
        match instruction {
            '>' => self.pointer = (self.pointer + 1) % self.tape_size,
            '<' => self.pointer = (self.pointer + self.tape_size - 1) % self.tape_size,
            '+' => self.tape[self.pointer] = (self.tape[self.pointer] + 1) % self.cell_size.as_u32(),
            '-' => {
                let cell_max = self.cell_size.as_u32();
                self.tape[self.pointer] = (self.tape[self.pointer] + cell_max - 1) % cell_max;
            }
            '[' => {
                if self.tape[self.pointer] == 0 {
                    if let Some(matching_pos) = self.loop_map.get(&Self::pos_to_key(&self.current_pos)).cloned() {
                        self.current_pos = matching_pos;
                    }
                }
            }
            ']' => {
                if self.tape[self.pointer] != 0 {
                    if let Some(matching_pos) = self.loop_map.get(&Self::pos_to_key(&self.current_pos)).cloned() {
                        self.current_pos = matching_pos;
                    }
                }
            }
            '.' => {
                if let Some(ch) = char::from_u32(self.tape[self.pointer]) {
                    self.output.push(ch);
                }
            }
            ',' => {
                console::log_1(&format!("Input requested at position {}", self.pointer).into());
            }
            _ => {}
        }
    }

    fn should_pause_at_breakpoint(&self, pos: &Position) -> bool {
        self.breakpoints.iter().any(|bp| bp.line == pos.line && bp.column == pos.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let mut interpreter = BrainfuckInterpreter::new(1024, 256).unwrap();
        let code = vec![
            Line { text: "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.".to_string() }
        ];
        
        interpreter.set_code(&serde_json::to_string(&code).unwrap()).unwrap();
        interpreter.run_turbo().unwrap();
        
        assert_eq!(interpreter.get_output(), "Hello World!\n");
    }
}