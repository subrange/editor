Pavlo, here’s the exact fix path and what to change, step-by-step. This removes the “\x01X / \x00X / XX” drift by making the caller and callee agree on how parameters are placed and read.

What’s actually broken
•	Your callee loads stack parameters from hard-coded FP - k slots that don’t match the actual placement for that signature.
•	Your caller and callee must agree on:
•	how many argument register words are used (A0..A3 = 4 words total),
•	which parameters spill to the stack,
•	and, for fat pointers spilled to the stack, the word order (address first, then bank).

Where to fix

1) Callee: compute stack offsets from actual placement

File: calling_convention.rs
Function: impl CallingConvention { pub fn load_param(&self, index: usize, param_types: &[(TempId, IrType)], mgr: &mut RegisterPressureManager, naming: &mut NameGenerator) -> (Vec<AsmInst>, Reg, Option<Reg>) }

Replace the body with logic that:
•	Models each param in words: scalar = 1, fat ptr = 2.
•	Packs up to 4 words into A0..A3 left-to-right; remainder go on the stack.
•	Computes stack offsets relative to FP using the saved area size (6 words: RA, FP, S0..S3).
•	Loads fat pointers on stack as addr (low word) at FP-… then bank (high word) at FP-…+1.
•	For register-resident params, move from A-regs into temps immediately; never read A0..A3 again inside the callee.
•	When loading a fat pointer (reg or stack), record its bank register in your RegisterPressureManager (so later GEP sees the bank as BankInfo::Register(reg)).

Pseudo-outline (drop this into the function body, adjusting names if your API differs):

// classify params into 1- or 2-word items
// decide A0..A3 packing (4 words budget)
// build list of stack params (left->right), compute FP-relative offsets:
//   FP-1 = nearest stack word, then walk downward
// convention for fat ptr on stack: [addr][bank] (low then high)
// load scalars/fat-ptrs accordingly:
//   - reg params: MOV from A-regs to temps
//   - stack params: compute FP+offset into SC, then LOAD
// record BankInfo::Register(bank_reg) for fat pointer temps

If helpful, I can supply a full ready-to-paste implementation matching your types; the core is as above.

2) Caller: push fat pointers as “addr then bank”

File: where you lower calls (often instruction.rs or builder.rs, the part that emits STORE …; ADDI SP, SP, 1 for stack args).

Two rules to enforce:
•	Packing rule (registers): treat A0..A3 as a 4-word window packed left-to-right. Each scalar consumes 1 word; each fat pointer consumes 2 contiguous words (addr then bank). Overflow goes to the stack.
•	Stack layout rule: push right-to-left, and for each fat pointer pushed, emit:

; fat ptr on stack — NEW order
STORE <addr>, SB, SP
ADDI  SP, SP, 1
STORE <bank>, SB, SP
ADDI  SP, SP, 1

Do not push bank first. The callee now expects addr at the lower address and bank at +1.

If your caller already follows this, great—just confirm the order. If it pushes bank then addr, flip it.

3) (Optional but wise) Codify “save-once” in the callee

File: function.rs (parameter binding loop)

Right after you call load_param and add the emitted instructions, add a brief comment/reminder:

// From here, never read A0..A3 directly in this function;
// load_param moved all register params into temps.

load_param should already move from A-regs into S-temps or locals; this note prevents accidental reuse.

⸻

Why this fixes your four test variants
•	minimal_insert(list, &dummy, 1, 'X')
Two fat-ptr words (list + dummy) consume A0..A3; scalars spill. The callee now loads pos/ch from correct FP offsets, so list[0] is untouched and list[1] becomes 'X'.
•	minimal_insert2(list, 1, 'X') and minimal_insert3(list, 'X')
Scalars in registers. The callee immediately moves them from A-regs to temps, so later code cannot clobber A-regs and smear values into your frame.
•	minimal_insert4(list)
Still works; nothing to fetch beyond the fat pointer.

⸻

Sanity checks you can run
1.	Build your four-function test again. Expect:

AB -> after minimal_insert -> AX

for the 2-char list demo.
2.	Disassemble the callee: you should see
•	For 4-arg version: LOAD for pos and ch from FP-… offsets (no fixed -7/-8),
•	For 3- and 2-arg versions: MOVE from appropriate A-regs into S-regs/temps at the top of the callee.
3.	Grep the call site when arguments overflow: the two STORE lines for a fat pointer must be addr first, then bank.

⸻

If you want me to wire in the exact code for load_param and the exact push sequence in your call-lowering function with concrete AsmInst lines, paste the snippets of those two spots and I’ll hand you a drop-in patch tailored to your names and types.