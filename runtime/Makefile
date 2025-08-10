# Ripple C Runtime Library Makefile

# Tools
RCC = ../target/release/rcc
RASM = ../src/ripple-asm/target/release/rasm
RLINK = ../src/ripple-asm/target/release/rlink

# Assembler settings
BANK_SIZE = 4096
MAX_IMMEDIATE = 65535

# Source files
C_SOURCES = putchar.c puts.c memset.c memcpy.c
ASM_SOURCES = crt0.asm

# Generated files
ASM_FILES = $(C_SOURCES:.c=.asm)
C_OBJ_FILES = $(C_SOURCES:.c=.pobj)
# Don't include crt0 in the runtime library - it should be linked separately
RUNTIME_OBJ_FILES = $(C_OBJ_FILES)
ALL_OBJ_FILES = $(C_OBJ_FILES) crt0.pobj
RUNTIME_LIB = libruntime.par

# Default target
all: $(RUNTIME_LIB)

# Compile C files to assembly
%.asm: %.c
	@echo "Compiling $< to assembly..."
	@$(RCC) compile $< -o $@

# Assemble C-generated assembly to object files
$(C_OBJ_FILES): %.pobj: %.asm
	@echo "Assembling $< to object..."
	@$(RASM) assemble $< -o $@ --bank-size $(BANK_SIZE) --max-immediate $(MAX_IMMEDIATE)

# Special rule for crt0.pobj (already in assembly)
crt0.pobj: crt0.asm
	@echo "Assembling crt0.asm to object..."
	@$(RASM) assemble crt0.asm -o crt0.pobj --bank-size $(BANK_SIZE) --max-immediate $(MAX_IMMEDIATE)

# Create runtime library archive (without crt0)
$(RUNTIME_LIB): $(RUNTIME_OBJ_FILES)
	@echo "Creating runtime library archive..."
	@$(RLINK) $(RUNTIME_OBJ_FILES) -f archive -o $(RUNTIME_LIB)

# Clean generated files (but not crt0.asm which is hand-written)
clean:
	rm -f $(ASM_FILES) *.pobj *.par *.bf *.bfm
	@echo "Cleaned runtime directory"

# Test target - build a test program with runtime
test: $(RUNTIME_LIB) crt0.pobj
	@echo "Building test program with runtime..."
	@$(RCC) compile ../c-test/tests-runtime/test_runtime_simple.c -o test_main.asm
	@$(RASM) assemble test_main.asm -o test_main.pobj --bank-size $(BANK_SIZE) --max-immediate $(MAX_IMMEDIATE)
	@echo "Linking crt0 + runtime archive + test program..."
	@$(RLINK) crt0.pobj $(RUNTIME_LIB) test_main.pobj -f macro --standalone -o test_standalone.bf
	@echo "Test program built: test_standalone.bf"
	@echo "Run with: bfm expand test_standalone.bf | bf"

.PHONY: all clean test