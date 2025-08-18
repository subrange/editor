//! Inline Assembly Support for V2 Backend
//! 
//! This module handles inline assembly statements with extended syntax:
//! asm("code" : outputs : inputs : clobbers)
//! 
//! The implementation follows GCC's extended inline assembly format with
//! proper register allocation, constraint parsing, and placeholder substitution.

use rcc_frontend::ir::{AsmOperandIR, Value};
use rcc_codegen::{AsmInst, Reg};
use rcc_common::{CompilerError, SourceLocation};
use crate::naming::NameGenerator;
use log::{debug, warn};
use std::collections::{HashMap, HashSet};
use crate::regmgmt::RegisterPressureManager;

/// Parsed constraint information
#[derive(Debug, Clone)]
pub struct ParsedConstraint {
    /// Is this an output constraint (=), input/output (+), or input
    pub is_output: bool,
    /// Is this a read-write constraint (+)
    pub is_read_write: bool,
    /// The constraint type: 'r' for register, 'm' for memory, 'i' for immediate
    pub constraint_type: char,
    /// For read-write constraints, the index of the tied input operand
    pub tied_to: Option<usize>,
}

/// Operand with allocated register and constraint info
#[derive(Debug)]
struct AllocatedOperand {
    /// The original operand from IR
    operand: AsmOperandIR,
    /// Parsed constraint information
    constraint: ParsedConstraint,
    /// Allocated register for this operand
    register: Reg,
    /// Original value name (for tracking)
    value_name: String,
}

/// Parse a constraint string like "=r", "+r", "r", "m", "i"
fn parse_constraint(constraint_str: &str) -> ParsedConstraint {
    let mut chars = constraint_str.chars();
    let first = chars.next().unwrap_or('r');
    
    let (is_output, is_read_write, constraint_type) = match first {
        '=' => {
            // Output constraint
            let ctype = chars.next().unwrap_or('r');
            (true, false, ctype)
        }
        '+' => {
            // Input/output (read-write) constraint
            let ctype = chars.next().unwrap_or('r');
            (true, true, ctype)
        }
        _ => {
            // Input constraint (or immediate/memory)
            (false, false, first)
        }
    };
    
    ParsedConstraint {
        is_output,
        is_read_write,
        constraint_type,
        tied_to: None,
    }
}

/// Get a set of clobbered registers from string names
#[allow(dead_code)]
fn parse_clobbers(clobbers: &[String]) -> HashSet<Reg> {
    let mut clobbered = HashSet::new();
    
    for clobber in clobbers {
        // Use the built-in from_str method from ripple_asm::Register
        if let Some(reg) = Reg::from_str(clobber) {
            clobbered.insert(reg);
        } else {
            warn!("Unknown clobber register: {clobber}");
        }
    }
    
    clobbered
}

/// Allocate registers for operands based on constraints
fn allocate_operand_registers(
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    outputs: &[AsmOperandIR],
    inputs: &[AsmOperandIR],
    _clobbers: &[String],
) -> Result<(Vec<AllocatedOperand>, Vec<AllocatedOperand>), CompilerError> {
    let mut allocated_outputs = Vec::new();
    let mut allocated_inputs = Vec::new();
    let mut register_map: HashMap<usize, Reg> = HashMap::new();
    
    // First pass: Process outputs and allocate registers for them
    for (idx, output) in outputs.iter().enumerate() {
        let constraint = parse_constraint(&output.constraint);
        
        // Validate constraint type
        if constraint.constraint_type != 'r' && constraint.constraint_type != 'm' {
            return Err(CompilerError::codegen_error(
                format!("Unsupported output constraint type: {}", constraint.constraint_type),
                SourceLocation::new_simple(0, 0),
            ));
        }
        
        // For register constraints, get a register from the manager
        if constraint.constraint_type == 'r' {
            // Get the value name for tracking
            let value_name = match &output.value {
                Value::Temp(t) => naming.temp_name(*t),
                _ => format!("asm_output_{idx}"),
            };
            
            // Get a register from the manager
            let reg = mgr.get_register(value_name.clone());
            register_map.insert(idx, reg);
            
            allocated_outputs.push(AllocatedOperand {
                operand: output.clone(),
                constraint: constraint.clone(),
                register: reg,
                value_name,
            });
        } else {
            // Memory constraint - not yet fully implemented
            return Err(CompilerError::codegen_error(
                "Memory constraints not yet implemented".to_string(),
                SourceLocation::new_simple(0, 0),
            ));
        }
    }
    
    // Second pass: Process inputs
    for (idx, input) in inputs.iter().enumerate() {
        // Check if this input is tied to an output
        let tied_output_idx = input.tied_to;
        
        if let Some(output_idx) = tied_output_idx {
            // This input is tied to an output, use the same register
            if output_idx >= allocated_outputs.len() {
                return Err(CompilerError::codegen_error(
                    format!("Invalid tied operand index: {output_idx}"),
                    SourceLocation::new_simple(0, 0),
                ));
            }
            
            let output_reg = allocated_outputs[output_idx].register;
            let value_name = match &input.value {
                Value::Temp(t) => naming.temp_name(*t),
                _ => format!("asm_input_{idx}"),
            };
            
            allocated_inputs.push(AllocatedOperand {
                operand: input.clone(),
                constraint: parse_constraint(&input.constraint),
                register: output_reg,
                value_name,
            });
        } else {
            // Regular input operand
            let constraint = parse_constraint(&input.constraint);
            
            if constraint.constraint_type == 'r' {
                let value_name = match &input.value {
                    Value::Temp(t) => naming.temp_name(*t),
                    Value::Constant(c) => naming.const_value(*c),
                    _ => format!("asm_input_{idx}"),
                };
                
                // Get a register from the manager
                let reg = mgr.get_register(value_name.clone());
                
                allocated_inputs.push(AllocatedOperand {
                    operand: input.clone(),
                    constraint,
                    register: reg,
                    value_name,
                });
            } else if constraint.constraint_type == 'i' {
                // Immediate constraint - these will be handled differently
                return Err(CompilerError::codegen_error(
                    "Immediate constraints not yet implemented".to_string(),
                    SourceLocation::new_simple(0, 0),
                ));
            } else if constraint.constraint_type == 'm' {
                // Memory constraint
                return Err(CompilerError::codegen_error(
                    "Memory constraints not yet implemented".to_string(),
                    SourceLocation::new_simple(0, 0),
                ));
            } else {
                return Err(CompilerError::codegen_error(
                    format!("Unsupported input constraint: {}", input.constraint),
                    SourceLocation::new_simple(0, 0),
                ));
            }
        }
    }
    
    Ok((allocated_outputs, allocated_inputs))
}

/// Substitute placeholders in assembly string with allocated registers
fn substitute_placeholders(
    assembly: &str,
    allocated_outputs: &[AllocatedOperand],
    allocated_inputs: &[AllocatedOperand],
) -> String {
    let mut result = assembly.to_string();
    
    // Replace %0, %1, %2, etc. with register names
    // Outputs come first in the numbering, then inputs
    
    // Process outputs
    for (idx, op) in allocated_outputs.iter().enumerate() {
        let placeholder = format!("%{idx}");
        let reg_name = format!("{:?}", op.register);
        result = result.replace(&placeholder, &reg_name);
    }
    
    // Process inputs (continuing the numbering after outputs)
    let output_count = allocated_outputs.len();
    for (idx, op) in allocated_inputs.iter().enumerate() {
        let placeholder = format!("%{}", output_count + idx);
        let reg_name = format!("{:?}", op.register);
        result = result.replace(&placeholder, &reg_name);
    }
    
    result
}

/// Generate setup code to load input values into allocated registers
fn generate_setup_code(
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    allocated_inputs: &[AllocatedOperand],
) -> Vec<AsmInst> {
    let mut insts = Vec::new();

    for op in allocated_inputs {
        let constraint = &op.constraint;
        if constraint.constraint_type != 'r' {
            // keep 'i'/'m' for later extension if you want
            continue;
        }

        match &op.operand.value {
            Value::Temp(t) => {
                let temp_name = naming.temp_name(*t);
                let src_reg = mgr.get_register(temp_name.clone());
                insts.extend(mgr.take_instructions());

                if src_reg != op.register {
                    insts.push(AsmInst::Comment(format!(
                        "Load input {} into {:?}", op.value_name, op.register
                    )));
                    insts.push(AsmInst::Move(op.register, src_reg));
                }
                // CRUCIAL: Don’t free src_reg; this temp may still be live.
                // Also, tell the manager that op.register now ALSO contains that value.
                mgr.bind_value_to_register(temp_name, op.register);
            }
            Value::Constant(c) => {
                insts.push(AsmInst::Comment(format!(
                    "Load constant {} into {:?}", c, op.register
                )));
                insts.push(AsmInst::Li(op.register, *c as i16));
                // Bind the constant pseudo-name so the allocator knows
                let cname = naming.const_value(*c);
                mgr.bind_value_to_register(cname, op.register);
            }
            _ => {
                warn!("Unsupported input value type: {:?}", op.operand.value);
            }
        }
    }

    insts
}

/// Generate teardown code to store output values from registers
fn generate_teardown_code(
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    allocated_outputs: &[AllocatedOperand],
) -> Result<Vec<AsmInst>, CompilerError> {
    let mut insts = Vec::new();

    for op in allocated_outputs {
        if op.constraint.constraint_type != 'r' {
            return Err(CompilerError::codegen_error(
                format!("Output constraint '{}' not yet supported", op.constraint.constraint_type),
                SourceLocation::new_simple(0, 0),
            ));
        }

        match &op.operand.value {
            Value::Temp(t) => {
                let temp_name = naming.temp_name(*t);
                // Tell the allocator: temp is in op.register now.
                mgr.bind_value_to_register(temp_name.clone(), op.register);
                insts.push(AsmInst::Comment(format!(
                    "Output {} now in {:?}", temp_name, op.register
                )));
                // No stores here. Normal codegen will use this binding.
            }
            other => {
                warn!("Unsupported output value target: {other:?}");
            }
        }
    }

    Ok(insts)
}

/// Lower extended inline assembly with operands
pub fn lower_inline_asm_extended(
    mgr: &mut RegisterPressureManager,
    naming: &mut NameGenerator,
    assembly: &str,
    outputs: &[AsmOperandIR],
    inputs: &[AsmOperandIR],
    clobbers: &[String],
) -> Result<Vec<AsmInst>, CompilerError> {
    debug!("Lowering extended inline assembly:");
    debug!("  Assembly: {assembly}");
    debug!("  Outputs: {outputs:?}");
    debug!("  Inputs: {inputs:?}");
    debug!("  Clobbers: {clobbers:?}");

    let mut insts = Vec::new();

    if outputs.is_empty() && inputs.is_empty() {
        for part in assembly.split([';', '\n']) {
            let t = part.trim();
            if !t.is_empty() { insts.push(AsmInst::Raw(t.to_string())); }
        }
        return Ok(insts);
    }

    insts.push(AsmInst::Comment("=== Begin inline assembly ===".to_string()));

    // Conservative but correct baseline:
    mgr.spill_all();
    insts.extend(mgr.take_instructions());

    // Allocate regs for operands (keep only 'r' for now)
    let (allocated_outputs, allocated_inputs) =
        allocate_operand_registers(mgr, naming, outputs, inputs, clobbers)?;

    // Load/move inputs into their assigned regs
    insts.push(AsmInst::Comment("Setup: Load inputs".to_string()));
    insts.extend(generate_setup_code(mgr, naming, &allocated_inputs));
    insts.extend(mgr.take_instructions());

    // Substitute placeholders and emit raw asm
    let substituted = substitute_placeholders(assembly, &allocated_outputs, &allocated_inputs);
    insts.push(AsmInst::Comment("Inline assembly code".to_string()));
    for part in substituted.split([';', '\n']) {
        let t = part.trim();
        if !t.is_empty() { insts.push(AsmInst::Raw(t.to_string())); }
    }

    // Bind outputs to temps (no stores)
    insts.push(AsmInst::Comment("Teardown: Bind outputs".to_string()));
    insts.extend(generate_teardown_code(mgr, naming, &allocated_outputs)?);
    insts.extend(mgr.take_instructions());

    insts.push(AsmInst::Comment("=== End inline assembly ===".to_string()));
    Ok(insts)
}

/// Lower basic inline assembly (no operands)
pub fn lower_inline_asm_basic(assembly: &str) -> Vec<AsmInst> {
    let mut insts = Vec::new();
    
    // Split by semicolons or newlines to handle both styles
    for part in assembly.split([';', '\n']) {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            insts.push(AsmInst::Raw(trimmed.to_string()));
        }
    }
    
    insts
}