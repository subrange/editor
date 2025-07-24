// ===== BASIC OPERATIONS =====

#define clear [-]
#define inc(n) {repeat(n, +)}
#define dec(n) {repeat(n, -)}
#define next(n) {repeat(n, >)}
#define prev(n) {repeat(n, <)}

// Aliases for readability
#define right(n) @next(n)
#define left(n) @prev(n)
#define zero @clear

// ===== MEMORY OPERATIONS =====

// Move value n cells to the right (destructive)
#define move(n) [-@next(n)+@prev(n)]

// Copy value n cells to the right (non-destructive)
#define copy(n) [-@next(n)+>+@prev(n)<]@next(n)[-@prev(n)+@next(n)]@prev(n)

// Swap current cell with next cell
#define swap >[-<+>]<[->+<]

// ===== LANE OPERATIONS (for your 5-lane architecture) =====

#define nextword @next(5)
#define prevword @prev(5)
#define lane(n) @next(n)
#define tolane(n) @lane(n)

// Go to specific lane from lane 0
#define value_lane      // Already at lane 0
#define type_lane @next(1)
#define jump_lane @next(2)
#define nav_lane @next(3)
#define scratch_lane @next(4)

// Return to lane 0 from any lane (uses nav lane)
#define reset_lane @nav_lane[-]@prev(3)+@next(3)@prev(3)[@prev(1)]

// ===== ARITHMETIC =====

// Add value from next cell to current cell
#define add_next >[-<+>]

// Subtract value from next cell from current cell
#define sub_next >[-<->]

// Multiply current cell by n (destroys current cell)
#define mul(n) [->{repeat(n, +)}<]>[-<+>]

// Set cell to specific value
#define set(n) @clear @inc(n)

// ===== CONTROL FLOW HELPERS =====

// If current cell is non-zero, execute then clear
#define if [
#define endif @clear]

// While current cell is non-zero
#define while [
#define endwhile ]

// Basic loop n times (requires empty cell to the right)
#define loop(n) @inc(n)[@dec(1)
#define endloop ]

// ===== COMMON PATTERNS =====

// Print ASCII character
#define print(c) @set(c).@clear

// Print newline
#define newline @print(10)

// Print space
#define space @print(32)

// Store value in cell n (from current position)
#define store(n) @copy(n)@clear

// Load value from cell n (to current position)
#define load(n) @clear@next(n)@copy(-n)@prev(n)

// ===== STACK OPERATIONS =====
// Assumes stack grows to the right, SP at fixed position

#define push >[-]<[->+<]>>
#define pop <<[->>+<<]>

// ===== STRING/OUTPUT MACROS =====

// Print "Hello World!"
#define hello_world @print(72)@print(101)@print(108)@print(108)@print(111)@space@print(87)@print(111)@print(114)@print(108)@print(100)@print(33)@newline

// ===== ADVANCED PATTERNS =====

// Find next zero cell to the right
#define find_zero [>]

// Find next non-zero cell to the right
#define find_nonzero [>]<[>>]

// Boolean NOT (0->1, non-zero->0)
#define not @inc(1)[@clear@dec(1)]

// Compare with next cell (result in current: 0 if equal, non-zero otherwise)
#define compare >[-<->]

// ===== DEBUG HELPERS =====

// Mark current position with value 255 (useful for debugging)
#define mark @set(255)

// Create a "breakpoint" (infinite loop)
#define break @inc(1)[]

// ===== YOUR VM HELPERS =====

// For your command-based VM
#define nop @set(1)
#define add_cmd @set(2)
#define sub_cmd @set(3)
#define jmp_cmd @set(4)
#define jz_cmd @set(5)

// Push command with two operands
#define cmd(op, a, b) @set(op)>@set(a)>@set(b)>

// ===== MATH OPERATIONS =====

// Increment by 10 (faster than inc(10))
#define inc10 ++++++++++

// Common small increments
#define inc2 ++
#define inc3 +++
#define inc4 ++++
#define inc5 +++++

// Powers of 2
#define pow2(n) @set(1){repeat(n, @add_next@copy(1))}@clear


// ====== Program ======

@set(65)

@hello_world

