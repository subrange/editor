use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use web_sys::console;

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
}

#[wasm_bindgen]
impl BrainfuckInterpreter {
    #[wasm_bindgen(constructor)]
    pub fn new(tape_size: usize, cell_size: u32) -> Result<BrainfuckInterpreter, JsValue> {
        console::log_1(&"Creating new Brainfuck interpreter in Rust".into());
        
        let cell_size = CellSize::from_u32(cell_size)
            .ok_or_else(|| JsValue::from_str("Invalid cell size"))?;
        
        let tape = vec![0u32; tape_size];
        
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
        console::log_1(&"Starting turbo execution...".into());
        self.is_running = true;
        self.is_paused = false;
        self.is_stopped = false;

        let mut pc = 0;
        let mut ops_executed = 0u64;
        
        while pc < self.compiled_ops.len() && self.is_running && !self.is_paused {
            pc = self.execute_optimized_op(pc);
            ops_executed += 1;
            
            if ops_executed % 10_000_000 == 0 {
                console::log_1(&format!("Executed {} operations", ops_executed).into());
            }
        }
        
        self.is_running = false;
        console::log_1(&format!("Turbo execution completed: {} operations", ops_executed).into());
        
        Ok(())
    }
    
    fn execute_optimized_op(&mut self, pc: usize) -> usize {
        match &self.compiled_ops[pc] {
            Operation::MoveRight(n) => {
                self.pointer = (self.pointer + n) % self.tape_size;
                pc + 1
            }
            Operation::MoveLeft(n) => {
                self.pointer = (self.pointer + self.tape_size - (n % self.tape_size)) % self.tape_size;
                pc + 1
            }
            Operation::Increment(n) => {
                let cell_max = self.cell_size.as_u32();
                // Use unsafe for better performance when we know pointer is valid
                unsafe {
                    let cell = self.tape.get_unchecked_mut(self.pointer);
                    *cell = (*cell + *n as u32) % cell_max;
                }
                pc + 1
            }
            Operation::Decrement(n) => {
                let cell_max = self.cell_size.as_u32();
                let n_mod = (*n % cell_max as usize) as u32;
                unsafe {
                    let cell = self.tape.get_unchecked_mut(self.pointer);
                    *cell = (*cell + cell_max - n_mod) % cell_max;
                }
                pc + 1
            }
            Operation::LoopStart(end_pc) => {
                unsafe {
                    if *self.tape.get_unchecked(self.pointer) == 0 {
                        *end_pc  // Jump to the closing bracket
                    } else {
                        pc + 1
                    }
                }
            }
            Operation::LoopEnd(start_pc) => {
                unsafe {
                    if *self.tape.get_unchecked(self.pointer) != 0 {
                        *start_pc  // Jump TO the opening bracket
                    } else {
                        pc + 1
                    }
                }
            }
            Operation::Output => {
                unsafe {
                    self.output.push(char::from_u32(*self.tape.get_unchecked(self.pointer)).unwrap_or('?'));
                }
                pc + 1
            }
            Operation::Input => {
                console::log_1(&format!("Input requested at position {}", self.pointer).into());
                pc + 1
            }
            Operation::SetZero => {
                unsafe {
                    *self.tape.get_unchecked_mut(self.pointer) = 0;
                }
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
        }

        let mut ops_in_batch = 0;
        
        while self.turbo_pc < self.compiled_ops.len() && self.is_running && !self.is_paused && ops_in_batch < batch_size {
            self.turbo_pc = self.execute_optimized_op(self.turbo_pc);
            self.turbo_ops_executed += 1;
            ops_in_batch += 1;
        }
        
        if self.turbo_pc >= self.compiled_ops.len() {
            self.is_running = false;
            console::log_1(&format!("Turbo execution completed: {} operations", self.turbo_ops_executed).into());
            Ok(false) // No more work to do
        } else {
            Ok(true) // More work remains
        }
    }

    #[wasm_bindgen]
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    #[wasm_bindgen]
    pub fn resume(&mut self) {
        if self.is_running && self.is_paused {
            self.is_paused = false;
        }
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
}

impl BrainfuckInterpreter {
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
        
        // First pass: convert to basic operations
        let mut basic_ops = Vec::new();
        let mut jump_stack = Vec::new();
        
        for line in &self.code {
            for ch in line.text.chars() {
                let op_index = basic_ops.len();
                
                match ch {
                    '>' => basic_ops.push(Operation::MoveRight(1)),
                    '<' => basic_ops.push(Operation::MoveLeft(1)),
                    '+' => basic_ops.push(Operation::Increment(1)),
                    '-' => basic_ops.push(Operation::Decrement(1)),
                    '[' => {
                        jump_stack.push(op_index);
                        basic_ops.push(Operation::LoopStart(0));
                    }
                    ']' => {
                        if let Some(start_index) = jump_stack.pop() {
                            basic_ops[start_index] = Operation::LoopStart(op_index);
                            basic_ops.push(Operation::LoopEnd(start_index));
                        }
                    }
                    '.' => basic_ops.push(Operation::Output),
                    ',' => basic_ops.push(Operation::Input),
                    _ => {}
                }
            }
        }
        
        // Second pass: optimize
        self.compiled_ops = self.optimize_operations(basic_ops);
        
        // Third pass: fix jump addresses after optimization
        self.fix_jump_addresses();
        
        console::log_1(&format!("Compiled {} operations", self.compiled_ops.len()).into());
    }
    
    fn optimize_operations(&self, ops: Vec<Operation>) -> Vec<Operation> {
        let mut optimized = Vec::new();
        let mut i = 0;
        
        while i < ops.len() {
            match &ops[i] {
                Operation::LoopStart(end) => {
                    // Check for [-] pattern (clear loop)
                    if i + 2 < ops.len() && *end == i + 2 {
                        if matches!(ops[i + 1], Operation::Decrement(1)) {
                            console::log_1(&format!("Optimizing [-] at position {}", i).into());
                            optimized.push(Operation::SetZero);
                            i += 3; // Skip the entire loop
                            continue;
                        }
                    }
                    
                    // For now, disable complex pattern detection until we fix it
                    // It's causing issues with complex programs like mandelbrot
                    /*
                    // Check for simple move patterns [->+<]
                    if let Some(pattern) = self.detect_move_pattern(&ops, i, *end) {
                        optimized.push(pattern);
                        i = *end + 1;
                        continue;
                    }
                    */
                    
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
        
        // Must decrement current cell by 1
        if movements.get(&0) != Some(&-1) {
            return None;
        }
        
        movements.remove(&0);
        
        // If only one other cell is modified by +1, it's a simple move
        if movements.len() == 1 {
            if let Some((&offset, &delta)) = movements.iter().next() {
                if delta == 1 {
                    return Some(Operation::MoveValue(offset));
                }
            }
        }
        
        // Otherwise, it's a multiply-move pattern
        if !movements.is_empty() {
            let mut moves: Vec<(isize, i32)> = movements.into_iter().collect();
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