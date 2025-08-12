//! Bank information management for pointers in V2 backend

use rcc_codegen::Reg;

/// Information about the bank register for a pointer
#[derive(Debug, Clone)]
pub enum BankInfo {
    /// Global bank (bank 0) - use R0
    Global,
    
    /// Stack bank (bank 1) - use R13 (must be initialized!)
    Stack,
    
    /// Dynamic bank in a register
    Register(Reg),
}

impl BankInfo {
    /// Get the register for this bank
    pub fn to_register(&self) -> Reg {
        match self {
            BankInfo::Global => Reg::R0,
            BankInfo::Stack => Reg::R13,
            BankInfo::Register(reg) => *reg,
        }
    }
    
    /// Check if this is a static bank (Global or Stack)
    pub fn is_static(&self) -> bool {
        matches!(self, BankInfo::Global | BankInfo::Stack)
    }
    
    /// Check if this requires R13 initialization
    pub fn requires_r13(&self) -> bool {
        matches!(self, BankInfo::Stack)
    }
}