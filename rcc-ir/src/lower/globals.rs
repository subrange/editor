use rcc_codegen::{AsmInst, Reg};
use rcc_common::CompilerError;
use crate::{GlobalVariable, IrType, Value};
use crate::module_lowering::ModuleLowerer;

impl ModuleLowerer {
    pub (crate) fn lower_global(&mut self, global: &GlobalVariable) -> Result<(), CompilerError> {
        // Check if this is a string literal (name starts with __str_)
        let is_string = global.name.starts_with("__str_");

        // Allocate address for global
        let address = self.next_global_addr;
        self.global_addresses.insert(global.name.clone(), address);

        // Calculate size in words (16-bit)
        let size = match &global.var_type {
            IrType::I8 | IrType::I16 => 1,
            IrType::I32 => 2, // 32-bit takes 2 words
            IrType::FatPtr(_) => 1, // Pointers are 16-bit
            IrType::Array { size, .. } if is_string => {
                // For strings, allocate one word per character (including null terminator)
                *size as u16  // Each character gets its own word
            }
            _ => 1, // Default to 1 word
        };

        self.next_global_addr += size;

        // For string literals, decode the string from the name
        if is_string {
            // Parse the hex-encoded string from the name
            // Format: __str_ID_HEXDATA
            if let Some(hex_part) = global.name.split('_').next_back() {
                let mut addr = address;
                let mut chars = Vec::new();

                // Decode hex string
                for i in (0..hex_part.len()).step_by(2) {
                    if let Ok(byte) = u8::from_str_radix(&hex_part[i..i + 2], 16) {
                        chars.push(byte);
                    }
                }
                chars.push(0); // Add null terminator

                // Create a safe string representation for the comment
                let safe_str: String = chars[..chars.len() - 1].iter()
                    .map(|&c| match c {
                        b'\n' => "\\n".to_string(),
                        b'\t' => "\\t".to_string(),
                        b'\r' => "\\r".to_string(),
                        b'\\' => "\\\\".to_string(),
                        c if c.is_ascii_graphic() || c == b' ' => (c as char).to_string(),
                        c => format!("\\x{c:02x}"),
                    })
                    .collect();

                self.emit(AsmInst::Comment(format!("String literal {safe_str} at address {address}")));
                
                // Store each character
                for byte in chars {
                    self.emit(AsmInst::LI(Reg::R3, byte as i16));
                    self.emit(AsmInst::LI(Reg::R4, addr as i16));
                    self.emit(AsmInst::Store(Reg::R3, Reg::R0, Reg::R4));
                    addr += 1;
                }
            }
        } else {
            self.emit(AsmInst::Comment(format!("Global variable: {} at address {}",
                                                            global.name, address)));

            // Generate initialization code if there's an initializer
            if let Some(init_value) = &global.initializer {
                match init_value {
                    Value::Constant(val) => {
                        // Load value into register and store at address
                        self.emit(AsmInst::LI(Reg::R3, *val as i16));
                        self.emit(AsmInst::LI(Reg::R4, address as i16));
                        self.emit(AsmInst::Store(Reg::R3, Reg::R0, Reg::R4));
                    }
                    _ => {
                        // Other initializer types not yet supported
                        self.emit(AsmInst::Comment(
                            format!("Unsupported initializer for {}", global.name)));
                    }
                }
            }
        }

        Ok(())
    }
}