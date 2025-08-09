use crate::encoder::InstructionEncoder;
use crate::parser::Parser;
use crate::types::*;
use crate::virtual_instructions::VirtualInstructionRegistry;
use std::collections::HashMap;

pub struct RippleAssembler {
    encoder: InstructionEncoder,
    parser: Parser,
    options: AssemblerOptions,
    virtual_registry: VirtualInstructionRegistry,
}

impl RippleAssembler {
    pub fn new(options: AssemblerOptions) -> Self {
        let encoder = InstructionEncoder::new(options.max_immediate);
        let parser = Parser::new(options.case_insensitive);
        let virtual_registry = VirtualInstructionRegistry::new();
        
        Self {
            encoder,
            parser,
            options,
            virtual_registry,
        }
    }

    pub fn assemble(&self, source: &str) -> Result<ObjectFile, Vec<String>> {
        let mut state = AssemblerState {
            current_bank: self.options.start_bank,
            current_offset: 0,
            labels: HashMap::new(),
            data_labels: HashMap::new(),
            pending_references: HashMap::new(),
            instructions: Vec::new(),
            memory_data: Vec::new(),
            errors: Vec::new(),
        };

        let all_lines = self.parser.parse_source(source);
        
        // Expand virtual instructions
        let expanded_lines = self.expand_virtual_instructions(&all_lines, &mut state);
        
        // Process sections and directives
        let code_lines = self.process_sections(&expanded_lines, &mut state);
        
        // Two-pass assembly
        if state.errors.is_empty() {
            self.first_pass(&code_lines, &mut state);
        }
        
        if state.errors.is_empty() {
            self.second_pass(&code_lines, &mut state);
        }

        if !state.errors.is_empty() {
            return Err(state.errors);
        }

        // Create object file with unresolved references
        let unresolved_references = state.pending_references
            .into_iter()
            .map(|(idx, pending)| {
                let ref_type = match pending.ref_type {
                    ReferenceType::Branch => "branch",
                    ReferenceType::Absolute => "absolute",
                    ReferenceType::Data => "data",
                };
                (idx, UnresolvedReference {
                    label: pending.label,
                    ref_type: ref_type.to_string(),
                })
            })
            .collect();

        let entry_point = state.labels.get("start").map(|_| "start".to_string())
            .or_else(|| state.labels.get("_start").map(|_| "_start".to_string()));
        
        Ok(ObjectFile {
            version: 1,
            instructions: state.instructions,
            data: state.memory_data,
            labels: state.labels,
            data_labels: state.data_labels,
            unresolved_references,
            entry_point,
        })
    }

    fn expand_virtual_instructions(
        &self,
        lines: &[ParsedLine],
        state: &mut AssemblerState,
    ) -> Vec<ParsedLine> {
        let mut expanded = Vec::new();

        for line in lines {
            if let Some(mnemonic) = &line.mnemonic {
                if let Some(virtual_inst) = self.virtual_registry.get(mnemonic) {
                    match virtual_inst.expand(&line.operands) {
                        Ok(expanded_lines) => {
                            for mut exp_line in expanded_lines {
                                exp_line.line_number = line.line_number;
                                expanded.push(exp_line);
                            }
                        }
                        Err(e) => {
                            state.errors.push(format!("Line {}: {}", line.line_number, e));
                            expanded.push(line.clone());
                        }
                    }
                } else {
                    expanded.push(line.clone());
                }
            } else {
                expanded.push(line.clone());
            }
        }

        expanded
    }

    fn process_sections(
        &self,
        lines: &[ParsedLine],
        state: &mut AssemblerState,
    ) -> Vec<ParsedLine> {
        let mut current_section = Section::Code;
        let mut code_lines = Vec::new();
        let mut current_data_offset = self.options.data_offset as u32;

        for line in lines {
            if let Some(directive) = &line.directive {
                match directive.as_str() {
                    "data" => {
                        current_section = Section::Data;
                        continue;
                    }
                    "code" | "text" => {
                        current_section = Section::Code;
                        continue;
                    }
                    _ if current_section == Section::Data && line.label.is_none() => {
                        let bytes_added = self.process_data_directive(line, state);
                        current_data_offset += bytes_added as u32;
                        continue;
                    }
                    _ => {}
                }
            }

            // Handle labels in data section
            if current_section == Section::Data && line.label.is_some() {
                let label = line.label.as_ref().unwrap();
                state.data_labels.insert(label.clone(), current_data_offset);

                // Process data directive after label if present
                if line.directive.is_some() {
                    let bytes_added = self.process_data_directive(line, state);
                    current_data_offset += bytes_added as u32;
                }
                continue;
            }

            if current_section == Section::Code {
                code_lines.push(line.clone());
            }
        }

        code_lines
    }

    fn process_data_directive(&self, line: &ParsedLine, state: &mut AssemblerState) -> usize {
        let directive = line.directive.as_ref().unwrap();
        let mut bytes_added = 0;

        match directive.as_str() {
            "byte" | "db" => {
                for arg in &line.directive_args {
                    if let Ok(value) = self.encoder.parse_immediate(arg) {
                        state.memory_data.push(value as u8);
                        bytes_added += 1;
                    }
                }
            }
            "word" | "dw" => {
                for arg in &line.directive_args {
                    if let Ok(value) = self.encoder.parse_immediate(arg) {
                        state.memory_data.push((value & 0xFF) as u8);
                        state.memory_data.push((value >> 8) as u8);
                        bytes_added += 2;
                    }
                }
            }
            "asciiz" | "string" => {
                for arg in &line.directive_args {
                    if arg.starts_with('"') && arg.ends_with('"') {
                        let text = &arg[1..arg.len()-1];
                        let mut chars = text.chars().peekable();
                        while let Some(ch) = chars.next() {
                            if ch == '\\' {
                                // Handle escape sequences
                                if let Some(&next_ch) = chars.peek() {
                                    chars.next(); // consume the escaped character
                                    let byte = match next_ch {
                                        'n' => b'\n',
                                        'r' => b'\r',
                                        't' => b'\t',
                                        '\\' => b'\\',
                                        '"' => b'"',
                                        '0' => b'\0',
                                        _ => next_ch as u8, // fallback for unknown escapes
                                    };
                                    state.memory_data.push(byte);
                                    bytes_added += 1;
                                }
                            } else {
                                state.memory_data.push(ch as u8);
                                bytes_added += 1;
                            }
                        }
                        // Add null terminator for asciiz
                        if directive.as_str() == "asciiz" {
                            state.memory_data.push(0);
                            bytes_added += 1;
                        }
                    }
                }
            }
            _ => {
                state.errors.push(format!(
                    "Line {}: Unknown data directive: {}",
                    line.line_number, directive
                ));
            }
        }

        bytes_added
    }

    fn first_pass(&self, lines: &[ParsedLine], state: &mut AssemblerState) {
        for line in lines {
            // Handle labels
            if let Some(label) = &line.label {
                let absolute_address = (state.current_bank as u32 * self.options.bank_size as u32) 
                    + state.current_offset as u32;
                
                state.labels.insert(label.clone(), Label {
                    name: label.clone(),
                    bank: state.current_bank,
                    offset: state.current_offset,
                    absolute_address,
                });
            }

            // Count instruction size
            if line.mnemonic.is_some() {
                state.current_offset += INSTRUCTION_SIZE;
                
                // Check for bank overflow
                if state.current_offset >= self.options.bank_size {
                    state.current_bank += 1;
                    state.current_offset = 0;
                }
            }
        }
    }

    fn second_pass(&self, lines: &[ParsedLine], state: &mut AssemblerState) {
        state.current_bank = self.options.start_bank;
        state.current_offset = 0;

        for line in lines {
            if let Some(mnemonic) = &line.mnemonic {
                // Special handling for HALT
                if mnemonic == "HALT" {
                    state.instructions.push(Instruction::new(Opcode::Nop, 0, 0, 0));
                } else if let Some(opcode) = Opcode::from_str(mnemonic) {
                    // Check if operands contain label references and replace them with placeholders
                    let mut modified_operands = line.operands.clone();
                    let mut has_label_ref = false;
                    
                    for (i, operand) in line.operands.iter().enumerate() {
                        // Check if operand is likely a label (not a register or immediate)
                        // Be case-insensitive for register checks
                        let upper_op = operand.to_uppercase();
                        let is_register = Register::from_str(&upper_op).is_some();
                        let is_number = operand.parse::<i32>().is_ok()
                            || operand.starts_with("0x") || operand.starts_with("0X")
                            || operand.starts_with("0b") || operand.starts_with("0B");
                        
                        if !is_register && !is_number {
                            // This is likely a label reference
                            has_label_ref = true;
                            // Replace with placeholder 0 - actual address will be resolved by linker
                            modified_operands[i] = "0".to_string();
                        }
                    }
                    
                    match self.encoder.encode(opcode, &modified_operands) {
                        Ok(instruction) => {
                            // Store label references if any
                            if has_label_ref {
                                self.check_label_references(&line.operands, state.instructions.len(), state, opcode);
                            }
                            state.instructions.push(instruction);
                        }
                        Err(e) => {
                            state.errors.push(format!("Line {}: {}", line.line_number, e));
                        }
                    }
                } else {
                    state.errors.push(format!(
                        "Line {}: Unknown instruction: {}",
                        line.line_number, mnemonic
                    ));
                }

                state.current_offset += INSTRUCTION_SIZE;
                if state.current_offset >= self.options.bank_size {
                    state.current_bank += 1;
                    state.current_offset = 0;
                }
            }
        }
    }

    fn check_label_references(
        &self,
        operands: &[String],
        instruction_idx: usize,
        state: &mut AssemblerState,
        opcode: Opcode,
    ) {
        for operand in operands {
            // Check if operand looks like a label (not a register or immediate)
            let upper_op = operand.to_uppercase();
            let is_register = Register::from_str(&upper_op).is_some();
            let is_number = operand.parse::<i32>().is_ok()
                || operand.starts_with("0x") || operand.starts_with("0X")
                || operand.starts_with("0b") || operand.starts_with("0B");
            
            if !is_register && !is_number {
                
                // Determine reference type based on the opcode
                let ref_type = match opcode {
                    Opcode::Beq | Opcode::Bne | Opcode::Blt | Opcode::Bge => ReferenceType::Branch,
                    Opcode::Jal => ReferenceType::Absolute,
                    Opcode::Load | Opcode::Store => ReferenceType::Data,
                    _ => ReferenceType::Absolute,
                };

                state.pending_references.insert(instruction_idx, PendingReference {
                    label: operand.clone(),
                    ref_type,
                });
            }
        }
    }

    pub fn assemble_to_binary(&self, source: &str) -> Result<Vec<u8>, Vec<String>> {
        let obj = self.assemble(source)?;
        
        // Convert to binary format
        let mut binary = Vec::new();
        
        // Write magic number
        binary.extend_from_slice(b"RASM");
        
        // Write version
        binary.extend_from_slice(&obj.version.to_le_bytes());
        
        // Write instruction count
        binary.extend_from_slice(&(obj.instructions.len() as u32).to_le_bytes());
        
        // Write instructions
        for inst in &obj.instructions {
            binary.push(inst.opcode);
            binary.push(inst.word0);
            binary.extend_from_slice(&inst.word1.to_le_bytes());
            binary.extend_from_slice(&inst.word2.to_le_bytes());
            binary.extend_from_slice(&inst.word3.to_le_bytes());
        }
        
        // Write data section size
        binary.extend_from_slice(&(obj.data.len() as u32).to_le_bytes());
        
        // Write data
        binary.extend_from_slice(&obj.data);
        
        Ok(binary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_nop() {
        let assembler = RippleAssembler::new(AssemblerOptions::default());
        let result = assembler.assemble("NOP").unwrap();
        
        assert_eq!(result.instructions.len(), 1);
        assert_eq!(result.instructions[0].opcode, Opcode::Nop as u8);
    }

    #[test]
    fn test_assemble_add() {
        let assembler = RippleAssembler::new(AssemblerOptions::default());
        let result = assembler.assemble("ADD R3, R4, R5").unwrap();
        
        assert_eq!(result.instructions.len(), 1);
        assert_eq!(result.instructions[0].opcode, Opcode::Add as u8);
        assert_eq!(result.instructions[0].word1, Register::R3 as u16);
        assert_eq!(result.instructions[0].word2, Register::R4 as u16);
        assert_eq!(result.instructions[0].word3, Register::R5 as u16);
    }

    #[test]
    fn test_assemble_with_label() {
        let assembler = RippleAssembler::new(AssemblerOptions::default());
        let source = r#"
start:
    LI R3, 42
    ADD R3, R3, R4
"#;
        let result = assembler.assemble(source).unwrap();
        
        assert_eq!(result.instructions.len(), 2);
        assert!(result.labels.contains_key("start"));
        assert_eq!(result.labels.get("start").unwrap().offset, 0);
    }

    #[test]
    fn test_assemble_data_section() {
        let assembler = RippleAssembler::new(AssemblerOptions::default());
        let source = r#"
.data
message: .asciiz "Hello"

.code
start:
    LI R3, 0
"#;
        let result = assembler.assemble(source).unwrap();
        
        assert_eq!(result.instructions.len(), 1);
        assert_eq!(&result.data[..], b"Hello\0");
        assert!(result.data_labels.contains_key("message"));
    }

    #[test]
    fn test_assemble_halt() {
        let assembler = RippleAssembler::new(AssemblerOptions::default());
        let result = assembler.assemble("HALT").unwrap();
        
        assert_eq!(result.instructions.len(), 1);
        assert!(result.instructions[0].is_halt());
    }
}