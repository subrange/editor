Short answer: don’t overload local_offsets. Track provenance (which bank a pointer points into) in a separate map, and consult that when emitting LOAD/STORE. GEP results stay as plain regs; you only carry a tag saying “this address is into stack” (or global).

Here’s a clean, robust way to do it.

1) Data you keep while lowering

// Already have:
local_offsets: HashMap<TempId, i32>   // only for direct Alloca results (frame slots)

// New:
#[derive(Clone, Copy, PartialEq, Eq)]
enum PtrRegion { Stack, Global, Unknown }

ptr_region: HashMap<TempId, PtrRegion>  // provenance tag for *pointer-valued* temps

Key rule:
•	local_offsets means “this temp is a frame slot with static offset from FP”.
•	ptr_region means “this temp is a pointer value in a register and that pointer points into Stack/Global”.

Do not stick GEP results into local_offsets ever.

2) Where you set the tag
   •	Alloca t
   You create a frame slot:

local_offsets.insert(t, frame_off);
ptr_region.insert(t, PtrRegion::Stack); // the *value* of alloca is a stack pointer


	•	&global → t (address-of global / constant string, etc.)

ptr_region.insert(t, PtrRegion::Global);


	•	GEP t = gep base, idx
	•	Compute base_reg from base (see §4).
	•	Emit ADD t_reg, base_reg, idx_reg/imm.
	•	Tag:

let reg = ptr_region.get(&base_tid)
.copied()
.or_else(|| if local_offsets.contains_key(&base_tid) { Some(PtrRegion::Stack) } else { None })
.unwrap_or(PtrRegion::Unknown);
ptr_region.insert(t, reg);


	•	PHI/Select producing a pointer t
	•	If all incoming tags are the same non-Unknown → that tag.
	•	If mixed or any Unknown → Unknown. (You can forbid this in M3, or later extend to carry a (bank,addr) pair.)
	•	Function args returning/receiving pointers
	•	If your ABI knows the bank (e.g., globals only), set tag on parameter temps at function entry. Otherwise mark Unknown for now.

3) Using the tag when emitting memory ops

Introduce a helper that materializes the address operand and chooses the bank:

/// Returns (addr_reg, bank_reg) ready for LOAD/STORE.
fn addr_and_bank(&mut self, p: Value) -> (Reg, Reg) {
match p {
// Direct alloca temp: compute FP+offset each time, bank=SB
Value::Temp(t) if self.local_offsets.contains_key(&t) => {
let off = self.local_offsets[ &t ];
let a = self.getReg();
self.emit(ADD(a, FP, R0));
if off != 0 { self.emit(ADDI(a, a, off)); }
return (a, SB); // SB = R13
}

        // General pointer temp in a register
        Value::Temp(t) => {
            let a = self.get_value_register(t); // <-- just the pointer register
            let bank = match self.ptr_region.get(&t).copied().unwrap_or(PtrRegion::Unknown) {
                PtrRegion::Stack  => SB, // R13
                PtrRegion::Global => GB, // R12
                PtrRegion::Unknown => {
                    // For M3 you can default to GB or assert. Better: error out until supported.
                    GB
                }
            };
            return (a, bank);
        }

        // Address-of global literal/label with constant offset
        Value::GlobalAddr(off) => {
            let a = self.getReg();
            self.emit(LI(a, off));
            return (a, GB);
        }

        // (Optional) address-of local, if you represent it explicitly
        Value::FrameAddr(off) => {
            let a = self.getReg();
            self.emit(ADD(a, FP, R0));
            if off != 0 { self.emit(ADDI(a, a, off)); }
            return (a, SB);
        }

        // Fallback
        _ => unreachable!("unexpected address value"),
    }
}

Then your STORE/LOAD lowering becomes:

// STORE src, p
let rs = get_value_register(src);
let (addr, bank) = addr_and_bank(p);
emit(STORE(rs, bank, addr));

// LOAD rd, p
let rd = getReg();
let (addr, bank) = addr_and_bank(p);
emit(LOAD(rd, bank, addr));

4) Getting a pointer into a register (don’t conflate with frame slots)

When an operand is a pointer:

fn get_pointer_in_reg(&mut self, v: Value) -> Reg {
match v {
// Direct alloca: *address* = FP+off (not the slot content)
Value::Temp(t) if self.local_offsets.contains_key(&t) => {
let off = self.local_offsets[ &t ];
let r = self.getReg();
emit(ADD(r, FP, R0));
if off != 0 { emit(ADDI(r, r, off)); }
// Tag stays Stack on the *temp*; this function only materializes the address
r
}
Value::Temp(t) => self.get_value_register(t), // previously computed (e.g., GEP)
Value::GlobalAddr(off) => {
let r = self.getReg();
emit(LI(r, off));
r
}
_ => unreachable!(),
}
}

Your GEP lowering uses that:

// t = gep base, idx
let base_r = get_pointer_in_reg(base);
let idx_r  = get_value_register(idx); // already scaled if needed
let t_r    = self.getReg();
emit(ADD(t_r, base_r, idx_r));
propagate_ptr_region(t /*from base*/);

Critical part: You do not write to local_offsets for t. That map remains exclusive to frame slots (allocas). t is just a pointer value in a reg, with a provenance tag in ptr_region.

5) Why your bug disappears

Previously you stuck the GEP result into local_offsets (with offset 0) as a marker. get_value_register then “helpfully” treated it like an alloca, recomputed FP+0, and you lost the actual address in R7. With the split:
•	local_offsets only for allocas.
•	ptr_region tells STORE/LOAD which bank to use, while the address operand is the register you computed in GEP.

So your line 58 becomes “use R7 as the addr reg, bank=SB”, not “ADD R8, R15, R0”.

6) Edge cases (what to do now vs later)
   •	PHI/Select mixing Stack & Global → mark Unknown. For M3: reject with a helpful error; later you can extend your IR to carry (bank, addr) pairs or split control-flow so each bank is uniform.
   •	Loading a pointer from memory: its provenance is whatever was stored there. If you need this, tag on STORE when the stored value is a pointer: if v has known ptr_region, remember it in a side table keyed by the (bank,addr) of the destination. For M3 you can skip and default to Global for loaded pointers.
   •	Element size ≠ 1: scale idx before ADD (SLI+ADDI) or constant fold.


