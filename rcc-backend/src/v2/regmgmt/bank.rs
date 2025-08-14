//! Bank information management for pointers in V2 backend

use rcc_codegen::Reg;

/// Information about the bank register for a pointer
#[derive(Debug, Clone)]
pub enum BankInfo {
    /// Global bank - use GP (R31)
    Global,
    
    /// Stack bank (bank 1) - use SB (R28, must be initialized!)
    Stack,
    
    /// Dynamic bank in a register
    Register(Reg),
    
    /// Dynamic bank in a named value (can be in register or spilled)
    /// The String is the value name that can be used with get_register()
    NamedValue(String),
}

impl BankInfo {
    /// Get the register for this bank (for static banks only)
    /// For NamedValue, this will panic - use get_bank_register_with_mgr instead
    pub fn to_register(&self) -> Reg {
        match self {
            BankInfo::Global => Reg::Gp,
            BankInfo::Stack => Reg::Sb,
            BankInfo::Register(reg) => *reg,
            BankInfo::NamedValue(name) => {
                panic!("Cannot get register for NamedValue('{}') without RegisterPressureManager - use get_bank_register_with_mgr", name)
            }
        }
    }
    
    /// Check if this is a static bank (Global or Stack)
    pub fn is_static(&self) -> bool {
        matches!(self, BankInfo::Global | BankInfo::Stack)
    }
    
    /// Check if this requires SB initialization
    pub fn requires_sb(&self) -> bool {
        matches!(self, BankInfo::Stack)
    }
}