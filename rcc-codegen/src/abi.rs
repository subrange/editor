//! Ripple VM ABI Implementation
//! 
//! This module implements the Application Binary Interface (ABI) for the Ripple VM,
//! including calling conventions, stack frame layout, and function prologue/epilogue generation.

use crate::asm::{AsmInst, Reg};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AbiError {
    #[error("Too many parameters: {0} (maximum: {1})")]
    TooManyParameters(usize, usize),
    
    #[error("Invalid register for parameter: {0:?}")]
    InvalidParameterRegister(Reg),
    
    #[error("Stack frame too large: {0} bytes")]
    FrameTooLarge(u16),
}

/// Ripple VM Calling Convention
/// 
/// Register Usage:
/// - R0: Zero/scratch register
/// - R3-R4: Scratch registers (R1-R2 don't exist)  
/// - R3-R8: Argument/return value registers
/// - R9-R12: Callee-saved registers
/// - R13: Stack bank register (always 0 for now)
/// - R14: Stack pointer
/// - R15: Frame pointer
/// - RA/RAB: Return address and bank
pub struct CallingConvention;

impl CallingConvention {
    /// Maximum number of parameters that can be passed in registers
    pub const MAX_REG_PARAMS: usize = 6; // R3-R8
    
    /// Registers used for passing parameters
    pub const PARAM_REGS: [Reg; 6] = [Reg::R3, Reg::R4, Reg::R5, Reg::R6, Reg::R7, Reg::R8];
    
    /// Registers that must be saved by callee
    pub const CALLEE_SAVED: [Reg; 4] = [Reg::R9, Reg::R10, Reg::R11, Reg::R12];
    
    /// Registers that can be freely used by callee
    pub const CALLER_SAVED: [Reg; 6] = [Reg::R0, Reg::R3, Reg::R4, Reg::R5, Reg::R6, Reg::R7];
    
    /// Stack registers
    pub const STACK_BANK: Reg = Reg::R13;  // Bank register for stack
    pub const STACK_PTR: Reg = Reg::R14;   // Stack pointer
    pub const FRAME_PTR: Reg = Reg::R15;   // Frame pointer
    
    /// Get the register for a parameter index (0-based)
    pub fn param_reg(index: usize) -> Result<Reg, AbiError> {
        if index >= Self::MAX_REG_PARAMS {
            return Err(AbiError::TooManyParameters(index + 1, Self::MAX_REG_PARAMS));
        }
        Ok(Self::PARAM_REGS[index])
    }
}

/// Stack Frame Layout
/// 
/// The Ripple VM uses a frame pointer-based stack that grows upward.
/// Each frame contains:
/// 1. Saved frame pointer
/// 2. Saved return address (if function makes calls)
/// 3. Saved callee registers
/// 4. Local variables
/// 5. Spilled temporaries
#[derive(Debug, Clone)]
pub struct Frame {
    /// Size of local variables in 16-bit words
    pub locals_size: u16,
    
    /// Callee-saved registers that need to be preserved
    pub saved_regs: Vec<Reg>,
    
    /// Whether this function makes calls (needs to save RA)
    pub has_calls: bool,
    
    /// Whether this function uses the frame pointer
    pub needs_frame_ptr: bool,
    
    /// Total frame size (computed)
    pub total_size: u16,
}

impl Frame {
    /// Create a new frame with the given local variables size
    pub fn new(locals_size: u16) -> Self {
        Self {
            locals_size,
            saved_regs: Vec::new(),
            has_calls: false,
            needs_frame_ptr: locals_size > 0,
            total_size: 0,
        }
    }
    
    /// Mark that this function makes calls
    pub fn set_has_calls(&mut self, has_calls: bool) {
        self.has_calls = has_calls;
        self.compute_frame_size();
    }
    
    /// Add a register that needs to be saved
    pub fn add_saved_reg(&mut self, reg: Reg) {
        if !self.saved_regs.contains(&reg) {
            self.saved_regs.push(reg);
            self.compute_frame_size();
        }
    }
    
    /// Compute the total frame size
    fn compute_frame_size(&mut self) {
        let mut size = 0;
        
        // Always save old frame pointer
        if self.needs_frame_ptr {
            size += 1;
        }
        
        // Save return address if making calls
        if self.has_calls {
            size += 1;
        }
        
        // Save callee registers
        size += self.saved_regs.len() as u16;
        
        // Local variables
        size += self.locals_size;
        
        self.total_size = size;
    }
    
    /// Generate function prologue
    /// 
    /// The prologue:
    /// 1. Saves the old frame pointer
    /// 2. Sets up the new frame pointer
    /// 3. Saves return address if needed
    /// 4. Saves callee registers
    /// 5. Allocates space for locals
    pub fn gen_prologue(&self) -> Vec<AsmInst> {
        let mut code = Vec::new();
        
        if !self.needs_frame_ptr && self.total_size == 0 {
            return code;
        }
        
        // Save old frame pointer
        if self.needs_frame_ptr {
            code.push(AsmInst::Store(
                CallingConvention::FRAME_PTR,
                CallingConvention::STACK_BANK,
                CallingConvention::STACK_PTR
            ));
            code.push(AsmInst::AddI(CallingConvention::STACK_PTR, CallingConvention::STACK_PTR, 1));
        }
        
        // Save return address if this function makes calls
        if self.has_calls {
            code.push(AsmInst::Store(
                Reg::RA,
                CallingConvention::STACK_BANK,
                CallingConvention::STACK_PTR
            ));
            code.push(AsmInst::AddI(CallingConvention::STACK_PTR, CallingConvention::STACK_PTR, 1));
        }
        
        // Save callee registers
        for &reg in &self.saved_regs {
            code.push(AsmInst::Store(
                reg,
                CallingConvention::STACK_BANK,
                CallingConvention::STACK_PTR
            ));
            code.push(AsmInst::AddI(CallingConvention::STACK_PTR, CallingConvention::STACK_PTR, 1));
        }
        
        // Set new frame pointer (points to start of local variables)
        if self.needs_frame_ptr {
            code.push(AsmInst::Add(
                CallingConvention::FRAME_PTR,
                CallingConvention::STACK_PTR,
                Reg::R0
            ));
        }
        
        // Allocate space for local variables
        if self.locals_size > 0 {
            code.push(AsmInst::AddI(
                CallingConvention::STACK_PTR,
                CallingConvention::STACK_PTR,
                self.locals_size as i16
            ));
        }
        
        code
    }
    
    /// Generate function epilogue
    /// 
    /// The epilogue:
    /// 1. Deallocates local variables
    /// 2. Restores callee registers
    /// 3. Restores return address if needed
    /// 4. Restores old frame pointer
    /// 5. Returns to caller
    pub fn gen_epilogue(&self) -> Vec<AsmInst> {
        let mut code = Vec::new();
        
        if !self.needs_frame_ptr && self.total_size == 0 {
            code.push(AsmInst::Ret);
            return code;
        }
        
        // Restore stack pointer to frame base
        if self.needs_frame_ptr {
            code.push(AsmInst::Add(
                CallingConvention::STACK_PTR,
                CallingConvention::FRAME_PTR,
                Reg::R0
            ));
        } else if self.locals_size > 0 {
            code.push(AsmInst::SubI(
                CallingConvention::STACK_PTR,
                CallingConvention::STACK_PTR,
                self.locals_size as i16
            ));
        }
        
        // Restore callee registers (in reverse order)
        for &reg in self.saved_regs.iter().rev() {
            code.push(AsmInst::SubI(CallingConvention::STACK_PTR, CallingConvention::STACK_PTR, 1));
            code.push(AsmInst::Load(
                reg,
                CallingConvention::STACK_BANK,
                CallingConvention::STACK_PTR
            ));
        }
        
        // Restore return address if saved
        if self.has_calls {
            code.push(AsmInst::SubI(CallingConvention::STACK_PTR, CallingConvention::STACK_PTR, 1));
            code.push(AsmInst::Load(
                Reg::RA,
                CallingConvention::STACK_BANK,
                CallingConvention::STACK_PTR
            ));
        }
        
        // Restore old frame pointer
        if self.needs_frame_ptr {
            code.push(AsmInst::SubI(CallingConvention::STACK_PTR, CallingConvention::STACK_PTR, 1));
            code.push(AsmInst::Load(
                CallingConvention::FRAME_PTR,
                CallingConvention::STACK_BANK,
                CallingConvention::STACK_PTR
            ));
        }
        
        // Return to caller
        code.push(AsmInst::Ret);
        
        code
    }
    
    /// Generate a call sequence to another function
    pub fn gen_call(&self, target: &str, args: &[Reg]) -> Result<Vec<AsmInst>, AbiError> {
        let mut code = Vec::new();
        
        if args.len() > CallingConvention::MAX_REG_PARAMS {
            return Err(AbiError::TooManyParameters(args.len(), CallingConvention::MAX_REG_PARAMS));
        }
        
        // Move arguments to parameter registers
        for (i, &arg_reg) in args.iter().enumerate() {
            let param_reg = CallingConvention::param_reg(i)?;
            if arg_reg != param_reg {
                code.push(AsmInst::Move(param_reg, arg_reg));
            }
        }
        
        // Call the function
        code.push(AsmInst::Call(target.to_string()));
        
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_calling_convention() {
        assert_eq!(CallingConvention::param_reg(0).unwrap(), Reg::R3);
        assert_eq!(CallingConvention::param_reg(5).unwrap(), Reg::R8);
        assert!(CallingConvention::param_reg(6).is_err());
    }

    #[test]
    fn test_simple_frame_prologue() {
        let frame = Frame::new(0);
        let prologue = frame.gen_prologue();
        
        // For a frame with no locals and no calls, should be empty
        assert!(prologue.is_empty());
    }

    #[test]
    fn test_frame_with_locals() {
        let mut frame = Frame::new(4); // 4 words of locals
        frame.set_has_calls(false);
        
        let prologue = frame.gen_prologue();
        
        // Should save FP, set new FP, allocate locals
        assert!(!prologue.is_empty());
        assert!(prologue.iter().any(|inst| matches!(inst, AsmInst::Store(Reg::R15, _, _))));
        assert!(prologue.iter().any(|inst| matches!(inst, AsmInst::AddI(Reg::R14, _, 4))));
    }

    #[test]
    fn test_frame_with_calls() {
        let mut frame = Frame::new(2);
        frame.set_has_calls(true);
        frame.add_saved_reg(Reg::R9);
        
        let prologue = frame.gen_prologue();
        let epilogue = frame.gen_epilogue();
        
        // Should save FP, RA, and R9
        assert!(prologue.iter().any(|inst| matches!(inst, AsmInst::Store(Reg::R15, _, _))));
        assert!(prologue.iter().any(|inst| matches!(inst, AsmInst::Store(Reg::RA, _, _))));
        assert!(prologue.iter().any(|inst| matches!(inst, AsmInst::Store(Reg::R9, _, _))));
        
        // Epilogue should restore in reverse order
        assert!(epilogue.iter().any(|inst| matches!(inst, AsmInst::Load(Reg::R9, _, _))));
        assert!(epilogue.iter().any(|inst| matches!(inst, AsmInst::Load(Reg::RA, _, _))));
        assert!(epilogue.iter().any(|inst| matches!(inst, AsmInst::Load(Reg::R15, _, _))));
        assert!(epilogue.iter().any(|inst| matches!(inst, AsmInst::Ret)));
    }

    #[test]
    fn test_call_generation() {
        let frame = Frame::new(0);
        let args = vec![Reg::R5, Reg::R6];
        let call_seq = frame.gen_call("printf", &args).unwrap();
        
        // Should move args to parameter registers and call
        assert!(call_seq.iter().any(|inst| matches!(inst, AsmInst::Move(Reg::R3, Reg::R5))));
        assert!(call_seq.iter().any(|inst| matches!(inst, AsmInst::Move(Reg::R4, Reg::R6))));
        assert!(call_seq.iter().any(|inst| matches!(inst, AsmInst::Call(label) if label == "printf")));
    }

    #[test]
    fn test_too_many_args() {
        let frame = Frame::new(0);
        let args: Vec<Reg> = (0..10).map(|i| match i {
            0 => Reg::R0, 1 => Reg::R3, 2 => Reg::R4, 3 => Reg::R5, 4 => Reg::R6,
            5 => Reg::R5, 6 => Reg::R6, 7 => Reg::R7, 8 => Reg::R8, _ => Reg::R9,
        }).collect();
        
        let result = frame.gen_call("func", &args);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AbiError::TooManyParameters(10, 6)));
    }
}