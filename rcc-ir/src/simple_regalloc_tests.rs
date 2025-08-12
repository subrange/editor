//! Unit tests for the simple register allocator

#[cfg(test)]
mod tests {
    use super::super::simple_regalloc::SimpleRegAlloc;
    use rcc_codegen::{AsmInst, Reg};

    #[test]
    fn test_basic_allocation() {
        let mut alloc = SimpleRegAlloc::new();
        
        // Should allocate R5 first (last in the free list)
        let r1 = alloc.get_reg("temp1".to_string());
        assert_eq!(r1, Reg::A0);
        
        // Should allocate R6 next
        let r2 = alloc.get_reg("temp2".to_string());
        assert_eq!(r2, Reg::A1);
        
        // Different register for different value
        assert_ne!(r1, r2);
    }

    #[test]
    fn test_reuse_same_value() {
        let mut alloc = SimpleRegAlloc::new();
        
        // Allocate for temp1
        let r1 = alloc.get_reg("temp1".to_string());
        
        // Asking for same value should return same register
        let r2 = alloc.get_reg("temp1".to_string());
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_free_and_realloc() {
        let mut alloc = SimpleRegAlloc::new();
        
        // Allocate R5
        let r1 = alloc.get_reg("temp1".to_string());
        assert_eq!(r1, Reg::A0);
        
        // Free it
        alloc.free_reg(r1);
        
        // Should be able to allocate it again for different value
        let r2 = alloc.get_reg("temp2".to_string());
        assert_eq!(r2, Reg::A0);
    }

    #[test]
    fn test_const_allocation() {
        let mut alloc = SimpleRegAlloc::new();
        
        // Get a register for a constant
        let r1 = alloc.get_const_reg(42);
        
        // Should generate LI instruction
        let instrs = alloc.take_instructions();
        assert_eq!(instrs.len(), 1);
        assert!(matches!(instrs[0], AsmInst::LI(_, 42)));
    }

    #[test]
    fn test_free_const_reg() {
        let mut alloc = SimpleRegAlloc::new();
        
        // Allocate for constant
        let r1 = alloc.get_const_reg(42);
        alloc.take_instructions(); // Clear instructions
        
        // Free it using const-specific method
        alloc.free_const_reg(r1);
        
        // Should be at front of free list (not back)
        // So next allocation should NOT be r1
        let r2 = alloc.get_reg("temp".to_string());
        assert_ne!(r2, r1);
    }

    #[test]
    fn test_exhaust_registers() {
        let mut alloc = SimpleRegAlloc::new();
        
        // Allocate all 7 registers (R5-R11)
        let mut regs = Vec::new();
        for i in 0..7 {
            let r = alloc.get_reg(format!("temp{}", i));
            regs.push(r);
        }
        
        // All should be different
        for i in 0..7 {
            for j in i+1..7 {
                assert_ne!(regs[i], regs[j]);
            }
        }
        
        // Next allocation should trigger spill
        let r8 = alloc.get_reg("temp7".to_string());
        
        // Should have generated spill instructions
        let instrs = alloc.take_instructions();
        assert!(instrs.iter().any(|i| matches!(i, AsmInst::Comment(s) if s.contains("Spilling"))));
        assert!(instrs.iter().any(|i| matches!(i, AsmInst::Store(_, _, _))));
    }

    #[test]
    fn test_spill_and_reload() {
        let mut alloc = SimpleRegAlloc::new();
        
        // Fill all registers with known values
        let mut allocated_values = Vec::new();
        for i in 0..7 {
            let val = format!("temp{}", i);
            allocated_values.push(val.clone());
            alloc.get_reg(val);
        }
        
        // This should spill one of the allocated values
        let _r_new = alloc.get_reg("temp_new".to_string());
        let spill_instrs = alloc.take_instructions();
        
        // Verify that something was spilled
        assert!(spill_instrs.iter().any(|i| matches!(i, AsmInst::Comment(s) if s.contains("Spilling"))));
        assert!(spill_instrs.iter().any(|i| matches!(i, AsmInst::Store(_, _, _))));
        
        // Find which value was spilled from the comment
        let spilled_value = spill_instrs.iter().find_map(|i| {
            if let AsmInst::Comment(s) = i {
                if s.starts_with("Spilling ") {
                    let parts: Vec<&str> = s.split(' ').collect();
                    if parts.len() > 1 {
                        return Some(parts[1].to_string());
                    }
                }
            }
            None
        }).expect("Should have found spilled value");
        
        // Now reload that specific value - should trigger reload
        let _r_reloaded = alloc.reload(spilled_value.clone());
        
        let reload_instrs = alloc.take_instructions();
        assert!(reload_instrs.iter().any(|i| matches!(i, AsmInst::Comment(s) if s.contains(&format!("Reloading {}", spilled_value)))));
        assert!(reload_instrs.iter().any(|i| matches!(i, AsmInst::Load(_, _, _))));
    }

    #[test]
    fn test_free_all() {
        let mut alloc = SimpleRegAlloc::new();
        
        // Allocate several registers
        let r1 = alloc.get_reg("temp1".to_string());
        let r2 = alloc.get_reg("temp2".to_string());
        let r3 = alloc.get_reg("temp3".to_string());
        
        // Free all
        alloc.free_all();
        
        // Should be able to allocate R5 again
        let r_new = alloc.get_reg("temp_new".to_string());
        assert_eq!(r_new, Reg::A0);
    }

    #[test]
    #[ignore]
    fn test_reset() {
        let mut alloc = SimpleRegAlloc::new();
        
        // Do some allocations and spills
        for i in 0..10 {
            alloc.get_reg(format!("temp{}", i));
        }
        
        // Reset
        alloc.reset();
        
        // Should be back to initial state
        let r1 = alloc.get_reg("temp_fresh".to_string());
        assert_eq!(r1, Reg::A0);
        
        // No spill instructions should be present
        let instrs = alloc.take_instructions();
        assert_eq!(instrs.len(), 0);
    }

    #[test]
    fn test_is_allocated() {
        let mut alloc = SimpleRegAlloc::new();
        
        let r1 = alloc.get_reg("temp1".to_string());
        assert!(alloc.is_allocated(r1));
        assert!(!alloc.is_allocated(Reg::A2)); // Not allocated yet
        
        alloc.free_reg(r1);
        assert!(!alloc.is_allocated(r1));
    }

    #[test]
    fn test_non_allocatable_registers() {
        let mut alloc = SimpleRegAlloc::new();
        
        // R3 and R4 are not in the allocatable pool
        // Freeing them should do nothing
        alloc.free_reg(Reg::Rv0);
        alloc.free_reg(Reg::Rv1);
        
        // R12-R15 are also not allocatable
        alloc.free_reg(Reg::Sc);
        alloc.free_reg(Reg::Sb);
        alloc.free_reg(Reg::Sp);
        alloc.free_reg(Reg::Fp);
        
        // Should still allocate R5 first
        let r1 = alloc.get_reg("temp1".to_string());
        assert_eq!(r1, Reg::A0);
    }

    #[test]
    fn test_spill_generates_correct_offset() {
        let mut alloc = SimpleRegAlloc::new();
        
        // Fill registers
        for i in 0..7 {
            alloc.get_reg(format!("temp{}", i));
        }
        
        // Cause multiple spills
        alloc.get_reg("spill1".to_string());
        let instrs1 = alloc.take_instructions();
        
        alloc.get_reg("spill2".to_string());
        let instrs2 = alloc.take_instructions();
        
        // Should have different spill offsets in comments
        let comment1 = instrs1.iter().find_map(|i| {
            if let AsmInst::Comment(s) = i {
                if s.contains("FP+") {
                    Some(s.clone())
                } else { None }
            } else { None }
        });
        
        let comment2 = instrs2.iter().find_map(|i| {
            if let AsmInst::Comment(s) = i {
                if s.contains("FP+") {
                    Some(s.clone())
                } else { None }
            } else { None }
        });
        
        assert!(comment1.is_some());
        assert!(comment2.is_some());
        assert_ne!(comment1, comment2); // Different offsets
    }

    #[test]
    fn test_reload_without_spill() {
        let mut alloc = SimpleRegAlloc::new();
        
        // Try to reload something that was never spilled
        let r = alloc.reload("never_seen".to_string());
        
        // Should just allocate a new register
        assert_eq!(r, Reg::A0);
        
        // No reload instructions should be generated
        let instrs = alloc.take_instructions();
        assert!(!instrs.iter().any(|i| matches!(i, AsmInst::Load(_, _, _))));
    }
}