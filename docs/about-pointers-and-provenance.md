Gotcha—let’s unpack what “PHI/Select mixing Stack & Global → mark Unknown” really means, how to implement it cleanly, and how to make M3 behave safely (error instead of silently generating wrong code). I’ll also sketch the next-step designs once you want this to “just work.”

⸻

Why this matters (tl;dr)

On Ripple you don’t have a single flat address space; every memory access needs a bank (Global vs Stack) plus an address. But your IR “pointer” is just a number; it doesn’t carry the bank. So when you combine pointers from different control-flow paths (PHI) or with a conditional (Select), you can end up with a value that might be a stack pointer on one path and a global pointer on another. If you lose that information, your LOAD/STORE pick the wrong bank and memory goes sideways.

⸻

A tiny lattice for pointer provenance

Track a tag for every pointer-valued temp:

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PtrRegion {
Unknown,  // we don’t know (yet) or can’t know
Stack,    // definitely into stack/FP bank
Global,   // definitely into global bank
Mixed,    // definitely can be Stack on some path and Global on others
}

Define a join (used by PHI/Select):

join(Unknown, X)   = X
join(X, Unknown)   = X
join(Stack, Stack) = Stack
join(Global,Global)= Global
join(Stack, Global)= Mixed
join(Global,Stack) = Mixed
join(Mixed, X)     = Mixed
join(X, Mixed)     = Mixed

This distinguishes “we don’t yet know” (Unknown) from “we know it’s path-dependent” (Mixed). For M3 you can error on either Unknown or Mixed at the point you need a concrete bank.

⸻

Propagation rules (where the tag comes from)

Maintain ptr_region: HashMap<TempId, PtrRegion>.
•	Alloca t
•	local_offsets[t] = frame_offset;
•	ptr_region[t] = Stack;            // alloca’s value is an address into the stack
•	Address-of global / string → t
•	ptr_region[t] = Global;
•	GEP t = gep base, idx
•	Emit arithmetic to compute t in a register (no local_offsets!)
•	ptr_region[t] = ptr_region[base] (default Unknown if base missing)
•	Copy/bitcast/move
•	ptr_region[dst] = ptr_region[src]
•	PHI t = φ(a,b,…)
•	ptr_region[t] = join_all(ptr_region[a], ptr_region[b], ...)
•	Select t = cond ? a : b
•	ptr_region[t] = join(ptr_region[a], ptr_region[b])
•	Function parameters (pointer-typed)
•	For M3: ptr_region[param] = Unknown (we don’t know caller’s bank).
•	Pointer results of calls
•	For M3: Unknown unless you annotate the callee (future: attributes).
•	Loading a pointer from memory
•	For M3: Unknown (unless you implement runtime tagging or a shadow table).

Important: Do not put GEP results into local_offsets. That map is for frame slots only.

⸻

Where you enforce it (M3)

When you need to choose a bank (i.e., before emitting LOAD rd, bank, addr or STORE rs, bank, addr):
1.	Materialize the address register (from a temp, or FP+offset if it’s a direct alloca).
2.	Look up PtrRegion:

let region = ptr_region.get(&temp).copied().unwrap_or(PtrRegion::Unknown);
match region {
PtrRegion::Stack  => bank = SB, // your stack bank register
PtrRegion::Global => bank = GB, // your global bank register
PtrRegion::Unknown | PtrRegion::Mixed => {
error_at(use_site,
"pointer provenance is unknown (bank cannot be determined)",
[
note(origin(temp)),
help("In M3, dereferencing pointers must have a known region."),
help("If this is a parameter, add a wrapper that fixes provenance or switch to a known bank."),
]);
}
}

Emit the error at the first dereference (or when passing to a callee that requires a known bank).

⸻

Why PHI/Select can become Mixed

Example:

int g;
int f(int *p) { return *p; }  // p’s bank depends on call site

int main() {
int x;
int *p;
if (cond) p = &x;    // Stack
else      p = &g;    // Global
return *p;           // PHI(p) mixes Stack and Global → Mixed → error in M3
}

Your propagation will compute Stack on the “then” path, Global on the “else” path, and Mixed at the join. In M3 you should reject this with a precise diagnostic (show both assignments with notes).

⸻

Your friend’s point (and what to do now)

Currently, for PtrRegion::Unknown, we default to global memory … This is why stack arrays passed to functions don’t work correctly.

Correct. Defaulting to Global silently breaks stack pointers. Fix for M3: never default; error out for Unknown/Mixed.

Add two small improvements:
•	Mark parameters Unknown, and error when dereferenced without establishing provenance.
•	Offer a tool-switch to unblock experiments: --assume-pointer-params=global|stack (still not correct, but explicit).

⸻

Future-proof paths beyond M3

Pick any (or several) of these when you want it to “just work”:

A) “Fat pointer” in IR (bank + addr)

Represent pointer values as a pair (addr, bank_tag) in SSA:
•	GEP moves addr, keeps bank_tag the same.
•	PHI/Select merge both fields independently (no information loss).
•	Calls pass both values.
•	LOAD/STORE use the bank field directly.
•	If you store pointers in memory, store both fields (or pack bank into a spare bit if your addr space allows).

This is the cleanest model.

B) Hidden bank parameter per pointer (ABI tweak; pragmatic)

For every pointer parameter, pass an extra hidden arg that carries the bank (e.g., 0=Global, 1=Stack). In the callee, set ptr_region[param] from that hidden value. This keeps IR flat pointers but fixes parameters and PHIs inside one function (since the bank is now a normal SSA value you can PHI right alongside the pointer).

Call-site lowering:

let (bank_tag, addr_reg) = classify_and_materialize(arg);
emit_pass(addr_reg);
emit_pass(bank_tag); // hidden

Callee prologue:

ptr_region[param_tid] = tag_from_hidden_arg; // exact, not Unknown

C) Function specialization by bank (no ABI change)

Compile two versions of any function that takes pointer parameters: foo$stack and foo$global. At each call site, pick the one that matches the caller’s known tag. If a call site’s tag is uncertain (e.g., PHI), error or insert a copy to a canonical region. Good for early performance; more compile-time work.

D) Shadow provenance (runtime tagging)

Keep a shadow map from (bank,addr) → bank-tag for pointers stored in memory (load retrieves tag). Useful if you need to load pointer values with correct bank later. More runtime cost; simpler compiler.

⸻

Practical M3 implementation notes
1.	Introduce Mixed now, even if you error on it; this lets you print better messages than a generic “Unknown”.
2.	Carry “origin info” with each tag (where set): for nice diagnostics.

struct Provenance {
region: PtrRegion,
origin: Span,      // first place we proved it
}
ptr_region: HashMap<TempId, Provenance>;

For PHI/Select, remember both contributing spans when you return Mixed so your error can show both branches.

	3.	Checkpoints to error:
	•	When lowering LOAD/STORE through a pointer with Unknown/Mixed.
	•	When passing a pointer arg and your ABI requires a known bank (for M3, you can allow passing but error on first deref in callee).
	4.	Message examples

error: pointer bank is path-dependent here (stack on one path, global on another)
--> foo.c:23:12
|
23 |     *p = 1;
|        ^
note: p points to local ‘x’ on this path
--> foo.c:19:14
note: p points to global ‘g’ on this path
--> foo.c:21:14
help: copy into a single region before dereferencing, e.g. move to a global buffer

Or for parameters:

error: cannot dereference parameter ‘p’: pointer bank unknown
--> bar.c:7:10
help: for M3, avoid passing addresses of locals; or compile with --assume-pointer-params=stack|global


⸻

Minimal code hooks (sketch)

fn join(a: PtrRegion, b: PtrRegion) -> PtrRegion {
use PtrRegion::*;
match (a,b) {
(Unknown, x) => x,
(x, Unknown) => x,
(Stack, Stack) => Stack,
(Global, Global) => Global,
_ => Mixed,
}
}

fn region_of_temp(&self, t: TempId) -> PtrRegion {
self.ptr_region.get(&t).map(|p| p.region).unwrap_or(PtrRegion::Unknown)
}

// PHI lowering:
let r = args.iter().map(|t| region_of_temp(*t)).reduce(join).unwrap_or(Unknown);
self.ptr_region.insert(result_tid, Provenance { region: r, origin: phi_span });

// LOAD/STORE:
let (addr, region) = self.addr_and_region(ptr_value);
match region {
PtrRegion::Stack  => emit(LOAD(rd, SB, addr)),
PtrRegion::Global => emit(LOAD(rd, GB, addr)),
PtrRegion::Unknown|PtrRegion::Mixed => error_at(span, "..."),
}


⸻

Bottom line
•	For M3: Track provenance; PHI/Select use join; error on Unknown/Mixed at deref; mark params Unknown; no more “default to global.”
•	Next: choose one of (A) fat pointers, (B) hidden bank parameter, or (C) specialization. Any of these will make PHIs and parameters robust without fragile heuristics.

If you want, point me at your phi and select lowerers plus your emit_load/store, and I’ll drop in the exact changes + diagnostics scaffolding.