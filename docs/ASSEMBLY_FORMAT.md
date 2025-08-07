Below is the full, up-to-date specification with a new § 11 Output-format examples that shows exactly how machine programs and data blobs are represented with the current macro set.

Save the whole file as ripple-vm-spec.md and treat it as canonical until the next change.

⸻

Ripple VM — Instruction-Set & System Specification

revision 2025-08-07-b

⸻

1. Fundamental data model

item	value	notes
Cell width	16-bit unsigned (0 … 65 535)
Instruction length	4 consecutive cells
Bank size	16 instructions (64 cells) — PC ∈ 0…15
Arithmetic	wrap-around modulo 65 536
Boolean	0 = false, 1 = true
R0	reads 0, writes are ignored


⸻

2. Register file

nr	name	purpose
0	R0	constant 0
1	PC	offset (0…15) within current bank
2	PCB	current bank number
3	RA	return address (low)
4	RAB	return bank  (high)
5 – 17	R3 … R15	general purpose


⸻

3. Program address

absoluteCell = PCB·64 + PC·4 + localWord   // localWord ∈ {0,1,2,3}

All jumps manipulate PC only (unless you load PCB explicitly).

⸻

4. Instruction word layout

word0  word1  word2  word3
┌────┬────┬────┬────┐
│opc │ a  │ b  │ c  │
└────┴────┴────┴────┘

class	words 1-3 mean	used by
R	rd, rs, rt	ALU ops, JALR
I	rd, rs, imm	ALU-imm, memory, branches
I1	rd, imm, 0	LI
I2	rd, imm1, imm2	JAL
J	addr, 0, 0	in-bank absolute jump


⸻

5. Opcode map & behaviour

5.1 ALU (R)

hex	mnemonic	effect
00	NOP	—
01	ADD	rd ← rs + rt
02	SUB	rd ← rs − rt
03	AND	rd ← rs & rt
04	OR	rd ← rs | rt
05	XOR	rd ← rs ^ rt
06	SLL	rd ← rs << (rt & 15)
07	SRL	rd ← rs >> (rt & 15)
08	SLT	signed compare
09	SLTU	unsigned compare

5.2 ALU-immediate (I / I1)

hex	mnemonic	effect
0A	ADDI	rd ← rs + imm
0B	ANDI	rd ← rs & imm (zero-ext)
0C	ORI	rd ← rs | imm (zero-ext)
0D	XORI	rd ← rs ^ imm (zero-ext)
0E	LI	rd ← imm (no shift)
0F	SLLI	rd ← rs << imm
10	SRLI	rd ← rs >> imm

5.3 Memory (I)

hex	mnemonic	effect
11	LOAD	rd ← MEM[rs + imm]
12	STORE	MEM[rs + imm] ← rd

5.4 Control flow

hex	form	effect
13	JAL addr	RA ← PC+1, RAB ← PCB, PC ← addr
14	JALR rd, 0, rs(assembler: JALR rd, rs)	rd ← PC+1, RAB ← PCB, PC ← rs
15	BEQ rs, rt, imm	if equal → PC+=imm
16	BNE rs, rt, imm	if not-equal
17	BLT rs, rt, imm	if signed less
18	BGE rs, rt, imm	if signed ≥
00	HALT	enter HALT state

All branch targets are bank-local; assembler emits a far jump as:

LI   PCB, bank(label)
JAL  addr(label)


⸻

6. Memory-mapped I/O

address	name	action
0x0000	OUT	write a byte → host stdout
0x0001	OUT_FLAG	host sets 1 when ready


⸻

7. Execution state machine

SETUP → RUNNING
while RUNNING:
fetch 4 words @ (PCB,PC)
execute
PC ← (PC+1) & 0xF
HALT ⇒ stop


⸻

8. Assembler rules
   •	Case-insensitive mnemonics.
   •	Immediates are unsigned 16-bit unless prefixed with -.
   •	JALR rd, rs ⇒ machine words opc=0x14, rd, 0, rs.
   •	Labels resolved per bank, far jumps auto-patched (see §5.4).

⸻

9. Reserved opcodes

0x19 … 0x1F are unused.

⸻

10. Change log
    •	2025-08-07-b – spec section 11 added, LI shift rule deleted, bank-local rule clarified.

⸻

11. Output-format examples

The custom pre-processor emits code with helper macros like @program_start, @cmd, @lane, etc.
These examples compile without manual bank management.

11.1 Simple countdown loop

@program_start(@OP_LI,     @R3, 5,   0)        // R3 ← 5
@cmd(          @OP_LI,     @R4, 3,   0)        // R4 ← 3  (loop entry)
@cmd(          @OP_JALR,   @R4, 0,   @R4)      // call loop

// ---- LOOP BODY (bank-local addr 3) ----
@cmd(          @OP_LI,     @R6, 1,   0)        // R6 ← 1
@cmd(          @OP_SUB,    @R3, @R3, @R6)      // R3 -= 1
@cmd(          @OP_ADD,    @R8, @R8, @R6)      // R8 += 1
@cmd(          @OP_BNE,    @R3, @R0, 1)        // skip HALT if R3 ≠ 0
@cmd(          @OP_HALT,   0,   0,   0)        // stop when done
@cmd(          @OP_JALR,   @RA, 0,   @R4)      // recurse
@cmd(          @OP_LI,     @R6, 42,  0)        // never executed
@program_end

11.2 Hello-world streamer

// Compile-time constant blob
#define HELLO {'H','e','l','l','o',',',' ','R','i','p','p','l','e',#ENDL}

// Data segment
@lane(#L_MEM,
{for(c in #HELLO, @set(c) @nextword)}
)

// Program
@program_start(@OP_LI,  #R3, 0, 0)        // loader
@cmd(          @OP_LI,  #R4, 4, 0)        // loop addr
@cmd(          @OP_LI,  #R5, 2, 0)        // mem pointer

@cmd(          @OP_JALR,#R4, 0, #R4)      // jump to body

// ---- BODY (addr 4) ----
@cmd(          @OP_LOAD, #R3, 0, #R5)     // R3 ← *R5
@cmd(          @OP_BNE,  #R3, 0, 1)       // EOF? -> halt
@cmd(          @OP_HALT, 0,   0,   0)

@cmd(          @OP_ADDI, #R5, #R5, 1)     // ++R5
@cmd(          @OP_STOR, #R3, 0,   0)     // OUT ← R3
@cmd(          @OP_JALR, #R4, 0, #R4)     // repeat
@program_end

Assembler responsibilities
•	Expand macro calls into the 4-word layout.
•	Ensure labels used by @program_start, @cmd, or branch pseudo-ops stay inside a bank, emitting auto-patches (LI PCB, imm) when necessary.
•	Pack literal blobs (HELLO) into consecutive @lane(#L_MEM, …) fragments.

⸻

End of document