#!/usr/bin/env python3
"""
Test runner for the Ripple C compiler
"""

import subprocess
import sys
import os
from pathlib import Path

# Build paths
RCC = "../target/release/rcc"
RASM = "../src/ripple-asm/target/release/rasm"
RLINK = "../src/ripple-asm/target/release/rlink"
RBT = "rbt"  # Global binary

# Runtime files
RUNTIME_DIR = "../runtime"
CRT0 = f"{RUNTIME_DIR}/crt0.pobj"
RUNTIME_OBJS = [
    f"{RUNTIME_DIR}/putchar.pobj",
    f"{RUNTIME_DIR}/puts.pobj",
    f"{RUNTIME_DIR}/memset.pobj",
    f"{RUNTIME_DIR}/memcpy.pobj"
]

def run_command(cmd, timeout=2):
    """Run a command with timeout"""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout)
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return -1, "", "Timeout"

def compile_and_run(c_file, expected_output, use_runtime=True):
    """Compile a C file and run it, checking output"""
    basename = Path(c_file).stem
    asm_file = f"{basename}.asm"
    pobj_file = f"{basename}.pobj"
    bf_file = f"{basename}.bf"
    
    # Clean up previous files
    for f in [asm_file, pobj_file, bf_file, f"{basename}_expanded.bf"]:
        if os.path.exists(f):
            os.remove(f)
    
    # Compile C to assembly
    ret, stdout, stderr = run_command(f"{RCC} compile {c_file} -o {asm_file}")
    if ret != 0:
        return False, f"Compilation failed: {stderr}"
    
    # Assemble to object
    ret, stdout, stderr = run_command(f"{RASM} assemble {asm_file} -o {pobj_file} --bank-size 32 --max-immediate 1000000")
    if ret != 0:
        return False, f"Assembly failed: {stderr}"
    
    if use_runtime:
        # Link with runtime
        objs = f"{CRT0} {' '.join(RUNTIME_OBJS)} {pobj_file}"
        ret, stdout, stderr = run_command(f"{RLINK} {objs} -f macro --standalone -o {bf_file}")
    else:
        # Use rbt for direct execution
        ret, stdout, stderr = run_command(f"gtimeout 2 {RBT} {asm_file} --run", timeout=2)
        if ret == -1:
            return False, "Execution timeout"
        if expected_output and stdout != expected_output:
            return False, f"Output mismatch. Expected: {repr(expected_output)}, Got: {repr(stdout)}"
        return True, "OK"
    
    if ret != 0:
        return False, f"Linking failed: {stderr}"
    
    # Expand and run
    expanded_file = f"{basename}_expanded.bf"
    ret, stdout, stderr = run_command(f"bfm expand {bf_file} -o {expanded_file}")
    if ret != 0:
        return False, f"Macro expansion failed: {stderr}"
    
    ret, stdout, stderr = run_command(f"bf {expanded_file}", timeout=2)
    if ret == -1:
        return False, "Execution timeout"
    
    if expected_output and stdout != expected_output:
        return False, f"Output mismatch. Expected: {repr(expected_output)}, Got: {repr(stdout)}"
    
    return True, "OK"

def main():
    # Ensure runtime is built
    print("Building runtime library...")
    ret, _, _ = run_command(f"cd {RUNTIME_DIR} && make clean && make all")
    if ret != 0:
        print("Failed to build runtime")
        return 1
    
    tests = [
        # Basic tests without runtime
        ("test_add.c", "Y\n", False),
        ("test_if_else_simple.c", "12", False),
        ("test_while_simple.c", "01234567890", False),
        
        # Tests with runtime
        ("test_runtime_simple.c", "RT", True),
        ("test_runtime_full.c", "Hi!\\nOK\\n", True),
        ("test_external_putchar.c", "X", True),
    ]
    
    passed = 0
    failed = 0
    
    print("\nRunning tests...")
    print("-" * 60)
    
    for test_file, expected, use_runtime in tests:
        if not os.path.exists(test_file):
            print(f"SKIP {test_file}: File not found")
            continue
            
        success, message = compile_and_run(test_file, expected, use_runtime)
        
        if success:
            print(f"✓ {test_file}")
            passed += 1
        else:
            print(f"✗ {test_file}: {message}")
            failed += 1
    
    print("-" * 60)
    print(f"Results: {passed} passed, {failed} failed")
    
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())