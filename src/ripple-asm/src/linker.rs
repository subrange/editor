use crate::types::{Instruction, Label, ObjectFile, Opcode, ReferenceType, UnresolvedReference};
use std::collections::HashMap;
use std::path::Path;
use std::fs;

pub struct Linker {
    bank_size: u16,
}

impl Linker {
    pub fn new(bank_size: u16) -> Self {
        Self { bank_size }
    }

    pub fn link(&self, object_files: Vec<ObjectFile>) -> Result<LinkedProgram, Vec<String>> {
        let mut errors = Vec::new();
        let mut all_instructions = Vec::new();
        let mut all_data = Vec::new();
        let mut global_labels = HashMap::new();
        let mut global_data_labels = HashMap::new();
        let mut instruction_offset = 0usize;
        let mut data_offset = 0u32;

        // Collect all instructions, data, and labels from all object files
        for (file_idx, obj) in object_files.iter().enumerate() {
            // Add instructions
            let file_instruction_start = all_instructions.len();
            all_instructions.extend_from_slice(&obj.instructions);

            // Add data
            let file_data_start = all_data.len() as u32;
            all_data.extend_from_slice(&obj.data);

            // Merge labels with offset adjustment
            for (name, label) in &obj.labels {
                let mut adjusted_label = label.clone();
                adjusted_label.absolute_address += instruction_offset as u32;
                
                if global_labels.insert(name.clone(), adjusted_label).is_some() {
                    errors.push(format!("Duplicate label '{}' in file {}", name, file_idx));
                }
            }

            // Merge data labels with offset adjustment
            for (name, offset) in &obj.data_labels {
                let adjusted_offset = offset + file_data_start;
                
                if global_data_labels.insert(name.clone(), adjusted_offset).is_some() {
                    errors.push(format!("Duplicate data label '{}' in file {}", name, file_idx));
                }
            }

            instruction_offset += obj.instructions.len() * 4; // Each instruction is 4 bytes
            data_offset += obj.data.len() as u32;
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Resolve all references
        let mut resolved_instructions = all_instructions.clone();
        for (file_idx, obj) in object_files.iter().enumerate() {
            let file_instruction_start = object_files.iter()
                .take(file_idx)
                .map(|o| o.instructions.len())
                .sum::<usize>();

            for (local_idx, unresolved) in &obj.unresolved_references {
                let global_idx = file_instruction_start + local_idx;
                
                if let Err(e) = self.resolve_reference(
                    &mut resolved_instructions[global_idx],
                    &unresolved,
                    &global_labels,
                    &global_data_labels,
                    global_idx,
                ) {
                    errors.push(format!("File {}, instruction {}: {}", file_idx, local_idx, e));
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Find entry point
        let entry_point = object_files.iter()
            .filter_map(|obj| obj.entry_point.as_ref())
            .next()
            .and_then(|name| global_labels.get(name))
            .map(|label| label.absolute_address)
            .unwrap_or(0);

        Ok(LinkedProgram {
            instructions: resolved_instructions,
            data: all_data,
            labels: global_labels,
            data_labels: global_data_labels,
            entry_point,
        })
    }

    fn resolve_reference(
        &self,
        instruction: &mut Instruction,
        reference: &UnresolvedReference,
        labels: &HashMap<String, Label>,
        data_labels: &HashMap<String, u32>,
        current_idx: usize,
    ) -> Result<(), String> {
        match reference.ref_type.as_str() {
            "branch" => {
                // For branch instructions, calculate relative offset
                let target_label = labels.get(&reference.label)
                    .ok_or_else(|| format!("Undefined label: {}", reference.label))?;
                
                let current_addr = current_idx as i32 * 4;
                let target_addr = target_label.absolute_address as i32;
                let offset = (target_addr - current_addr) / 4;
                
                // Update the immediate field (word3 for branches)
                instruction.word3 = offset as u16;
            }
            "absolute" => {
                // For absolute references (JAL, etc.), use the absolute address
                if let Some(label) = labels.get(&reference.label) {
                    let addr = label.absolute_address;
                    // Split address into high and low parts
                    instruction.word2 = (addr >> 16) as u16;
                    instruction.word3 = (addr & 0xFFFF) as u16;
                } else if let Some(&data_addr) = data_labels.get(&reference.label) {
                    // Reference to data section
                    instruction.word2 = (data_addr >> 16) as u16;
                    instruction.word3 = (data_addr & 0xFFFF) as u16;
                } else {
                    return Err(format!("Undefined label: {}", reference.label));
                }
            }
            "data" => {
                // For data references (LOAD/STORE with labels)
                let data_addr = data_labels.get(&reference.label)
                    .ok_or_else(|| format!("Undefined data label: {}", reference.label))?;
                
                instruction.word3 = *data_addr as u16;
            }
            _ => {
                return Err(format!("Unknown reference type: {}", reference.ref_type));
            }
        }
        
        Ok(())
    }

    pub fn link_files(&self, paths: &[&Path]) -> Result<LinkedProgram, Vec<String>> {
        let mut object_files = Vec::new();
        let mut errors = Vec::new();

        for path in paths {
            match fs::read_to_string(path) {
                Ok(content) => {
                    match serde_json::from_str::<ObjectFile>(&content) {
                        Ok(obj) => object_files.push(obj),
                        Err(e) => errors.push(format!("Failed to parse {}: {}", path.display(), e)),
                    }
                }
                Err(e) => errors.push(format!("Failed to read {}: {}", path.display(), e)),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        self.link(object_files)
    }
}

#[derive(Debug)]
pub struct LinkedProgram {
    pub instructions: Vec<Instruction>,
    pub data: Vec<u8>,
    pub labels: HashMap<String, Label>,
    pub data_labels: HashMap<String, u32>,
    pub entry_point: u32,
}

impl LinkedProgram {
    pub fn to_binary(&self) -> Vec<u8> {
        let mut binary = Vec::new();
        
        // Write magic number for linked program
        binary.extend_from_slice(b"RLINK");
        
        // Write entry point
        binary.extend_from_slice(&self.entry_point.to_le_bytes());
        
        // Write instruction count
        binary.extend_from_slice(&(self.instructions.len() as u32).to_le_bytes());
        
        // Write instructions
        for inst in &self.instructions {
            binary.push(inst.opcode);
            binary.push(inst.word0);
            binary.extend_from_slice(&inst.word1.to_le_bytes());
            binary.extend_from_slice(&inst.word2.to_le_bytes());
            binary.extend_from_slice(&inst.word3.to_le_bytes());
        }
        
        // Write data section size
        binary.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        
        // Write data
        binary.extend_from_slice(&self.data);
        
        binary
    }

    pub fn to_text(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("; Linked Program\n"));
        output.push_str(&format!("; Entry point: 0x{:08X}\n", self.entry_point));
        output.push_str(&format!("; Instructions: {}\n", self.instructions.len()));
        output.push_str(&format!("; Data size: {} bytes\n\n", self.data.len()));
        
        // Output instructions with addresses
        for (idx, inst) in self.instructions.iter().enumerate() {
            let addr = idx * 4;
            output.push_str(&format!(
                "{:08X}: {:02X} {:02X} {:04X} {:04X} {:04X}\n",
                addr, inst.opcode, inst.word0, inst.word1, inst.word2, inst.word3
            ));
        }
        
        // Output data section
        if !self.data.is_empty() {
            output.push_str("\n; Data Section:\n");
            for (idx, chunk) in self.data.chunks(16).enumerate() {
                output.push_str(&format!("{:08X}: ", idx * 16));
                for byte in chunk {
                    output.push_str(&format!("{:02X} ", byte));
                }
                output.push_str("\n");
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::RippleAssembler;
    use crate::types::AssemblerOptions;

    #[test]
    fn test_link_single_file() {
        let assembler = RippleAssembler::new(AssemblerOptions::default());
        let source = r#"
start:
    LI R3, 42
    HALT
"#;
        let obj = assembler.assemble(source).unwrap();
        
        let linker = Linker::new(16);
        let linked = linker.link(vec![obj]).unwrap();
        
        assert_eq!(linked.instructions.len(), 2);
        assert_eq!(linked.entry_point, 0);
    }

    #[test]
    fn test_link_multiple_files() {
        let assembler = RippleAssembler::new(AssemblerOptions::default());
        
        let source1 = r#"
start:
    JAL RA, R0, func
    HALT
"#;
        let obj1 = assembler.assemble(source1).unwrap();
        
        let source2 = r#"
func:
    LI R3, 42
    RET
"#;
        let obj2 = assembler.assemble(source2).unwrap();
        
        let linker = Linker::new(16);
        let linked = linker.link(vec![obj1, obj2]).unwrap();
        
        assert_eq!(linked.instructions.len(), 4); // JAL + HALT + LI + RET
        assert!(linked.labels.contains_key("start"));
        assert!(linked.labels.contains_key("func"));
    }
}