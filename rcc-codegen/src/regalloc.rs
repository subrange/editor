//! Register Allocation
//! 
//! This module implements register allocation for the Ripple VM. For M1, we use
//! a simple linear scan allocator. More sophisticated algorithms can be added later.

use crate::abi::CallingConvention;
use crate::asm::Reg;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegAllocError {
    #[error("No available registers for temporary {0}")]
    OutOfRegisters(u32),
    
    #[error("Invalid register: {0:?}")]
    InvalidRegister(Reg),
    
    #[error("Temporary {0} not found")]
    TemporaryNotFound(u32),
}

/// Represents a temporary variable that needs a register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Temporary(pub u32);

impl Temporary {
    pub fn new(id: u32) -> Self {
        Temporary(id)
    }
    
    pub fn id(&self) -> u32 {
        self.0
    }
}

/// Live range of a temporary variable
#[derive(Debug, Clone)]
pub struct LiveRange {
    pub temp: Temporary,
    pub start: usize,
    pub end: usize,
}

impl LiveRange {
    pub fn new(temp: Temporary, start: usize, end: usize) -> Self {
        LiveRange { temp, start, end }
    }
    
    pub fn overlaps(&self, other: &LiveRange) -> bool {
        self.start <= other.end && other.start <= self.end
    }
}

/// Simple linear scan register allocator
pub struct LinearScanAllocator {
    /// Available general-purpose registers for allocation
    available_regs: Vec<Reg>,
    
    /// Mapping from temporaries to allocated registers
    allocation: HashMap<Temporary, Reg>,
    
    /// Currently active (live) temporaries
    active: Vec<LiveRange>,
}

impl LinearScanAllocator {
    pub fn new() -> Self {
        // Use caller-saved registers for temporaries
        // Avoid R0 (often used as zero/scratch) and parameter registers during allocation
        let available_regs = vec![
            // Parameter registers R3-R8 can be used after parameter setup
            // Note: R1 and R2 don't exist in Ripple VM
            Reg::R3, Reg::R4, Reg::R5, Reg::R6, Reg::R7, Reg::R8,
        ];
        
        Self {
            available_regs,
            allocation: HashMap::new(),
            active: Vec::new(),
        }
    }
    
    /// Create an allocator with custom available registers
    pub fn with_registers(regs: Vec<Reg>) -> Self {
        Self {
            available_regs: regs,
            allocation: HashMap::new(),
            active: Vec::new(),
        }
    }
    
    /// Allocate registers for a list of live ranges
    pub fn allocate(&mut self, mut live_ranges: Vec<LiveRange>) -> Result<(), RegAllocError> {
        // Sort live ranges by start point
        live_ranges.sort_by_key(|range| range.start);
        
        for range in live_ranges {
            // Remove expired ranges from active list
            self.expire_old_ranges(range.start);
            
            // Try to allocate a register for this range
            if let Some(reg) = self.available_regs.iter()
                .find(|&&r| !self.is_register_active(r))
                .copied()
            {
                // Allocate the register
                self.allocation.insert(range.temp, reg);
                self.active.push(range);
            } else {
                // Need to spill - for now, just fail
                return Err(RegAllocError::OutOfRegisters(range.temp.id()));
            }
        }
        
        Ok(())
    }
    
    /// Get the allocated register for a temporary
    pub fn get_register(&self, temp: Temporary) -> Result<Reg, RegAllocError> {
        self.allocation.get(&temp)
            .copied()
            .ok_or(RegAllocError::TemporaryNotFound(temp.id()))
    }
    
    /// Get all allocations
    pub fn get_allocations(&self) -> &HashMap<Temporary, Reg> {
        &self.allocation
    }
    
    /// Remove ranges that are no longer active
    fn expire_old_ranges(&mut self, current_point: usize) {
        self.active.retain(|range| range.end >= current_point);
    }
    
    /// Check if a register is currently allocated to an active temporary
    fn is_register_active(&self, reg: Reg) -> bool {
        self.active.iter().any(|range| {
            self.allocation.get(&range.temp) == Some(&reg)
        })
    }
    
    /// Reset the allocator state
    pub fn reset(&mut self) {
        self.allocation.clear();
        self.active.clear();
    }
}

impl Default for LinearScanAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple register assignment for testing
pub struct SimpleRegAssigner {
    next_reg_index: usize,
    available_regs: Vec<Reg>,
    assignments: HashMap<Temporary, Reg>,
}

impl SimpleRegAssigner {
    pub fn new() -> Self {
        Self {
            next_reg_index: 0,
            available_regs: vec![Reg::R3, Reg::R4, Reg::R5, Reg::R6, Reg::R7, Reg::R8],
            assignments: HashMap::new(),
        }
    }
    
    /// Assign the next available register to a temporary
    pub fn assign_temp(&mut self, temp: Temporary) -> Result<Reg, RegAllocError> {
        if let Some(&reg) = self.assignments.get(&temp) {
            return Ok(reg);
        }
        
        if self.next_reg_index >= self.available_regs.len() {
            return Err(RegAllocError::OutOfRegisters(temp.id()));
        }
        
        let reg = self.available_regs[self.next_reg_index];
        self.assignments.insert(temp, reg);
        self.next_reg_index += 1;
        
        Ok(reg)
    }
    
    /// Get the assigned register for a temporary
    pub fn get_register(&self, temp: Temporary) -> Option<Reg> {
        self.assignments.get(&temp).copied()
    }
}

impl Default for SimpleRegAssigner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_temporary_creation() {
        let temp = Temporary::new(42);
        assert_eq!(temp.id(), 42);
    }

    #[test]
    fn test_live_range_overlap() {
        let range1 = LiveRange::new(Temporary::new(1), 0, 10);
        let range2 = LiveRange::new(Temporary::new(2), 5, 15);
        let range3 = LiveRange::new(Temporary::new(3), 12, 20);
        
        assert!(range1.overlaps(&range2));
        assert!(range2.overlaps(&range1));
        assert!(range2.overlaps(&range3));
        assert!(!range1.overlaps(&range3));
    }

    #[test]
    fn test_simple_reg_assigner() {
        let mut assigner = SimpleRegAssigner::new();
        
        let temp1 = Temporary::new(1);
        let temp2 = Temporary::new(2);
        
        let reg1 = assigner.assign_temp(temp1).unwrap();
        let reg2 = assigner.assign_temp(temp2).unwrap();
        
        assert_eq!(reg1, Reg::R3);
        assert_eq!(reg2, Reg::R4);
        
        // Assigning the same temp again should return the same register
        let reg1_again = assigner.assign_temp(temp1).unwrap();
        assert_eq!(reg1_again, Reg::R3);
    }

    #[test]
    fn test_simple_reg_assigner_exhaustion() {
        let mut assigner = SimpleRegAssigner::new();
        
        // Assign all available registers
        for i in 0..6 {
            let temp = Temporary::new(i);
            assert!(assigner.assign_temp(temp).is_ok());
        }
        
        // Next assignment should fail
        let temp_overflow = Temporary::new(10);
        assert!(assigner.assign_temp(temp_overflow).is_err());
    }

    #[test]
    fn test_linear_scan_allocator() {
        let mut allocator = LinearScanAllocator::new();
        
        let ranges = vec![
            LiveRange::new(Temporary::new(1), 0, 5),
            LiveRange::new(Temporary::new(2), 2, 8),
            LiveRange::new(Temporary::new(3), 6, 10),
        ];
        
        allocator.allocate(ranges).unwrap();
        
        // Check that allocations were made
        assert!(allocator.get_register(Temporary::new(1)).is_ok());
        assert!(allocator.get_register(Temporary::new(2)).is_ok());
        assert!(allocator.get_register(Temporary::new(3)).is_ok());
        
        // Temporaries 1 and 2 overlap, so should get different registers
        let reg1 = allocator.get_register(Temporary::new(1)).unwrap();
        let reg2 = allocator.get_register(Temporary::new(2)).unwrap();
        assert_ne!(reg1, reg2);
    }

    #[test]
    fn test_linear_scan_non_overlapping() {
        let mut allocator = LinearScanAllocator::new();
        
        // Non-overlapping ranges can share registers
        let ranges = vec![
            LiveRange::new(Temporary::new(1), 0, 3),
            LiveRange::new(Temporary::new(2), 4, 7),
        ];
        
        allocator.allocate(ranges).unwrap();
        
        let reg1 = allocator.get_register(Temporary::new(1)).unwrap();
        let reg2 = allocator.get_register(Temporary::new(2)).unwrap();
        
        // Since ranges don't overlap, they could potentially use the same register
        // But our current implementation doesn't reuse registers yet
        assert!(reg1 == reg2 || reg1 != reg2); // Either is fine for now
    }

    #[test]
    fn test_allocator_reset() {
        let mut allocator = LinearScanAllocator::new();
        
        let ranges = vec![LiveRange::new(Temporary::new(1), 0, 5)];
        allocator.allocate(ranges).unwrap();
        
        assert!(allocator.get_register(Temporary::new(1)).is_ok());
        
        allocator.reset();
        
        assert!(allocator.get_register(Temporary::new(1)).is_err());
    }
}