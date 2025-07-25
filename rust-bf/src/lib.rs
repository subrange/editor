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
}

#[derive(Debug, Clone)]
enum Operation {
    MoveRight,
    MoveLeft,
    Increment,
    Decrement,
    LoopStart(usize),
    LoopEnd(usize),
    Output,
    Input,
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
            match &self.compiled_ops[pc] {
                Operation::MoveRight => {
                    self.pointer = (self.pointer + 1) % self.tape_size;
                }
                Operation::MoveLeft => {
                    self.pointer = (self.pointer + self.tape_size - 1) % self.tape_size;
                }
                Operation::Increment => {
                    self.tape[self.pointer] = (self.tape[self.pointer] + 1) % self.cell_size.as_u32();
                }
                Operation::Decrement => {
                    let cell_max = self.cell_size.as_u32();
                    self.tape[self.pointer] = (self.tape[self.pointer] + cell_max - 1) % cell_max;
                }
                Operation::LoopStart(end_pc) => {
                    if self.tape[self.pointer] == 0 {
                        pc = *end_pc;
                    }
                }
                Operation::LoopEnd(start_pc) => {
                    if self.tape[self.pointer] != 0 {
                        pc = *start_pc;
                    }
                }
                Operation::Output => {
                    self.output.push(char::from_u32(self.tape[self.pointer]).unwrap_or('?'));
                }
                Operation::Input => {
                    console::log_1(&format!("Input requested at position {}", self.pointer).into());
                }
            }
            
            pc += 1;
            ops_executed += 1;
            
            if ops_executed % 10_000_000 == 0 {
                console::log_1(&format!("Executed {} operations", ops_executed).into());
            }
        }
        
        self.is_running = false;
        console::log_1(&format!("Turbo execution completed: {} operations", ops_executed).into());
        
        Ok(())
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
        let mut jump_stack = Vec::new();

        for line in &self.code {
            for ch in line.text.chars() {
                let op_index = self.compiled_ops.len();
                
                match ch {
                    '>' => self.compiled_ops.push(Operation::MoveRight),
                    '<' => self.compiled_ops.push(Operation::MoveLeft),
                    '+' => self.compiled_ops.push(Operation::Increment),
                    '-' => self.compiled_ops.push(Operation::Decrement),
                    '[' => {
                        jump_stack.push(op_index);
                        self.compiled_ops.push(Operation::LoopStart(0));
                    }
                    ']' => {
                        if let Some(start_index) = jump_stack.pop() {
                            self.compiled_ops[start_index] = Operation::LoopStart(op_index);
                            self.compiled_ops.push(Operation::LoopEnd(start_index));
                        }
                    }
                    '.' => self.compiled_ops.push(Operation::Output),
                    ',' => self.compiled_ops.push(Operation::Input),
                    _ => {}
                }
            }
        }
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