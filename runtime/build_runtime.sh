#!/bin/bash
# Build script to combine runtime library into a single assembly file

echo "; Ripple C Runtime Library" > runtime.asm
echo "; Combined runtime functions" >> runtime.asm
echo "" >> runtime.asm

# Add putchar (skip the _start and main sections)
echo "; putchar function" >> runtime.asm
grep -A 100 "^putchar:" putchar.asm | grep -B 100 "^RET" | head -n -1 >> runtime.asm
echo "" >> runtime.asm

# Add puts (skip the _start section)
echo "; puts function" >> runtime.asm
grep -A 100 "^puts:" puts.asm | grep -B 100 "^RET" >> runtime.asm
echo "" >> runtime.asm

# Add memset
echo "; memset function" >> runtime.asm
grep -A 100 "^memset:" memset.asm | grep -B 100 "^RET" >> runtime.asm
echo "" >> runtime.asm

# Add memcpy
echo "; memcpy function" >> runtime.asm
grep -A 100 "^memcpy:" memcpy.asm | grep -B 100 "^RET" >> runtime.asm

echo "Runtime library built: runtime.asm"