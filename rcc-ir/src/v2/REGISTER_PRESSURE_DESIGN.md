# Register Pressure Management Design for V2 Backend

## Problem Analysis

### The Core Issue
When lowering IR instructions to assembly, we face several register pressure scenarios:

1. **Binary Operations**: Need 2 registers (can reuse lhs for result via in-place ops)
2. **Complex Expressions**: Nested operations create temporary values
3. **Function Calls**: Need to save live values across calls (all registers are caller-saved)
4. **Fat Pointers**: Require 2 registers (addr + bank)
5. **Array Access**: Need registers for base, index, computed address
6. **Long-lived Values**: Variables used across many instructions

With only 7 allocatable registers (R5-R11), we quickly run out and need constant spilling/reloading.

### Current Approach Problems
1. **Ad-hoc Management**: Each instruction handler manages registers individually
2. **Redundant Spills**: Same value might be spilled/reloaded multiple times
3. **No Liveness Analysis**: Can't make optimal spilling decisions
4. **No Register Coalescing**: Can't reuse registers for non-overlapping values

## Proposed Solution: Centralized Register Pressure Manager

### Architecture

```
┌─────────────────────────────────────────┐
│          IR Instruction Stream          │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│       Register Pressure Manager         │
│  ┌──────────────────────────────────┐  │
│  │   Liveness Analysis Component    │  │
│  └──────────────────────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │   Value Lifetime Tracker         │  │
│  └──────────────────────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │   Spill Cost Calculator          │  │
│  └──────────────────────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │   Register Assignment Strategy   │  │
│  └──────────────────────────────────┘  │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│         RegAllocV2 (Low Level)          │
└─────────────────────────────────────────┘
```

## Implementation Design

### 1. Value Lifetime Tracking

```rust
/// Tracks the lifetime of values in a basic block
pub struct ValueLifetime {
    /// Instruction index where value is defined
    pub def_point: usize,
    
    /// Last instruction index where value is used
    pub last_use: usize,
    
    /// All instruction indices where value is used
    pub use_points: Vec<usize>,
    
    /// Whether this value crosses a function call
    pub crosses_call: bool,
    
    /// Estimated spill cost (based on loop depth, use frequency)
    pub spill_cost: f32,
}
```

### 2. Register Pressure Points

```rust
/// Identifies points of maximum register pressure
pub struct PressurePoint {
    /// Instruction index
    pub index: usize,
    
    /// Number of live values at this point
    pub live_count: usize,
    
    /// Values that are live at this point
    pub live_values: HashSet<TempId>,
    
    /// Suggested values to spill
    pub spill_candidates: Vec<TempId>,
}
```

### 3. Register Assignment Plan

```rust
/// Pre-computed register assignment for a basic block
pub struct RegisterPlan {
    /// Map from TempId to register assignment
    pub assignments: HashMap<TempId, RegisterAssignment>,
    
    /// Points where spills occur
    pub spill_points: Vec<(usize, TempId)>,
    
    /// Points where reloads occur
    pub reload_points: Vec<(usize, TempId)>,
}

pub enum RegisterAssignment {
    /// Value always in this register
    Fixed(Reg),
    
    /// Value in register during these ranges
    Ranges(Vec<(usize, usize, Reg)>),
    
    /// Value always spilled
    Spilled(i16), // Stack offset
}
```

### 4. Main Register Pressure Manager

```rust
pub struct RegisterPressureManager {
    /// Pre-analyzed lifetime information
    lifetimes: HashMap<TempId, ValueLifetime>,
    
    /// Pressure points in the block
    pressure_points: Vec<PressurePoint>,
    
    /// Pre-computed register plan
    plan: Option<RegisterPlan>,
    
    /// Underlying allocator
    allocator: RegAllocV2,
}

impl RegisterPressureManager {
    /// Analyze a basic block and prepare register plan
    pub fn analyze_block(&mut self, block: &BasicBlock) -> Result<(), String> {
        // 1. Build lifetime information
        self.build_lifetimes(block)?;
        
        // 2. Identify pressure points
        self.identify_pressure_points(block)?;
        
        // 3. Create register assignment plan
        self.plan = Some(self.create_register_plan()?);
        
        Ok(())
    }
    
    /// Get register for a value at specific instruction
    pub fn get_register(&mut self, temp_id: TempId, instr_index: usize) -> Reg {
        // Use pre-computed plan to minimize spills
        if let Some(plan) = &self.plan {
            // ... use plan to get optimal register
        } else {
            // Fall back to basic allocator
            self.allocator.get_reg(format!("t{}", temp_id))
        }
    }
    
    /// Smart spill decision based on liveness
    pub fn smart_spill(&mut self, at_index: usize) -> Option<Reg> {
        // Find best candidate to spill based on:
        // - Next use distance
        // - Spill cost
        // - Whether value crosses calls
    }
}
```

## Usage Pattern in IR Lowering

```rust
impl FunctionLowering {
    pub fn lower_basic_block(&mut self, block: &BasicBlock) -> Vec<AsmInst> {
        let mut rpm = RegisterPressureManager::new();
        
        // Pre-analyze the entire block
        rpm.analyze_block(block)?;
        
        let mut instructions = Vec::new();
        
        for (index, ir_inst) in block.instructions.iter().enumerate() {
            match ir_inst {
                Instruction::Binary { result, op, lhs, rhs, .. } => {
                    // RPM handles all register allocation optimally
                    let lhs_reg = rpm.get_value_register(lhs, index);
                    let rhs_reg = rpm.get_value_register(rhs, index);
                    let result_reg = rpm.get_register(*result, index);
                    
                    // Generate the instruction
                    instructions.push(emit_binary_op(*op, result_reg, lhs_reg, rhs_reg));
                    
                    // RPM handles any necessary spills/reloads
                    instructions.extend(rpm.take_spill_reload_instructions());
                }
                // ... other instructions
            }
        }
        
        instructions
    }
}
```

## Key Benefits

### 1. **Global Optimization**
- Sees entire basic block before making decisions
- Can minimize total spills through better planning

### 2. **Reduced Redundancy**
- Values spilled once and reloaded only when needed
- No duplicate spills of the same value

### 3. **Better Spill Decisions**
- Spills values with longest reuse distance
- Keeps frequently used values in registers

### 4. **Simplified IR Lowering**
- Each instruction handler just asks for registers
- RPM handles all complexity internally

### 5. **Improved Code Quality**
- Fewer spills = faster code
- Better register utilization

## Implementation Phases

### Phase 1: Basic Lifetime Analysis
- Track def-use chains
- Identify live ranges
- Calculate basic pressure points

### Phase 2: Smart Spilling
- Implement spill cost model
- Use next-use distance for decisions
- Handle call-crossing values

### Phase 3: Advanced Optimizations
- Register coalescing
- Live range splitting
- Rematerialization of constants

### Phase 4: Loop Awareness
- Higher spill cost for values in loops
- Loop-invariant code motion hints

## Example: Complex Expression

```c
// C code
int result = (a + b) * (c - d) + (e * f);
```

### Without RPM (Current Approach)
```
; Many redundant spills as each operation manages its own registers
LOAD a -> R5
LOAD b -> R6
ADD R7 = R5 + R6   ; might spill a or b
LOAD c -> R5       ; reuse R5
LOAD d -> R6       ; reuse R6
SUB R8 = R5 - R6   ; might spill R7
MUL R5 = R7 * R8   ; might spill both
LOAD e -> R6
LOAD f -> R7
MUL R8 = R6 * R7
ADD result = R5 + R8
```

### With RPM (Optimized)
```
; RPM pre-analyzes and creates optimal plan
LOAD a -> R5
LOAD b -> R6
ADD R7 = R5 + R6
LOAD c -> R8       ; RPM knows R8 available
LOAD d -> R9       ; RPM knows R9 available
SUB R10 = R8 - R9
MUL R5 = R7 * R10  ; Reuse R5 (a no longer needed)
LOAD e -> R6       ; Reuse R6 (b no longer needed)
LOAD f -> R7       ; Reuse R7 (first ADD result consumed)
MUL R6 = R6 * R7
ADD result = R5 + R6
```

## Metrics for Success

1. **Spill Reduction**: 30-50% fewer spills on average
2. **Code Size**: 10-20% reduction in generated instructions
3. **Clarity**: Simpler IR lowering code
4. **Maintainability**: Centralized register management

## Conclusion

The Register Pressure Manager provides a robust, centralized solution to register allocation challenges in the V2 backend. By pre-analyzing basic blocks and making globally optimal decisions, it significantly reduces spills and improves code quality while simplifying the IR lowering implementation.