#!/usr/bin/env python3
"""
Test runner for the Ripple C compiler

Usage: python3 run_tests.py [options]

Options:
  --no-cleanup  Don't clean up generated files after tests
  --clean       Only clean the build directory and exit

Prerequisites:
- Build the compiler: cargo build --release (in project root)
- Build the assembler: cargo build --release (in src/ripple-asm)
- Install required tools: rbt, gtimeout (brew install coreutils), bfm, bf

The script will:
1. Build the runtime library (crt0.pobj and libruntime.par)
2. Run tests with and without runtime linking
3. Report pass/fail status for each test
4. Clean up build artifacts (unless --no-cleanup is specified)
"""

import subprocess
import sys
import os
import glob
from pathlib import Path

# Colors for output
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
NC = '\033[0m'  # No Color

# Build paths
RCC = "../target/release/rcc"
RASM = "../src/ripple-asm/target/release/rasm"
RLINK = "../src/ripple-asm/target/release/rlink"
RBT = "rbt"  # Global binary

# Runtime files
RUNTIME_DIR = "../runtime"
CRT0 = f"{RUNTIME_DIR}/crt0.pobj"
RUNTIME_LIB = f"{RUNTIME_DIR}/libruntime.par"

# Build directory for temporary artifacts
BUILD_DIR = "build"

# Assembler settings (must match Makefile)
BANK_SIZE = 4096
MAX_IMMEDIATE = 65535

def run_command(cmd, timeout=5):
    """Run a command with timeout"""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout)
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return -1, "", "Timeout"

def compile_and_run(c_file, expected_output, use_full_runtime=True):
    """Compile a C file and run it, checking output
    
    Args:
        c_file: The C source file to compile
        expected_output: Expected output from the program
        use_full_runtime: If True, link with full runtime (crt0 + libruntime.par)
                         If False, link with crt0 only (for stack setup)
    
    Returns:
        tuple: (success, message, has_provenance_warning)
    """
    basename = Path(c_file).stem
    asm_file = f"{BUILD_DIR}/{basename}.asm"
    pobj_file = f"{BUILD_DIR}/{basename}.pobj"
    bf_file = f"{BUILD_DIR}/{basename}.bf"
    
    # Clean up previous files
    for f in [asm_file, pobj_file, bf_file, f"{BUILD_DIR}/{basename}_expanded.bf"]:
        if os.path.exists(f):
            os.remove(f)
    
    # Compile C to assembly
    ret, stdout, stderr = run_command(f"{RCC} compile {c_file} -o {asm_file}")
    if ret != 0:
        return False, f"Compilation failed: {stderr}", False
    
    # Check for provenance warnings in the generated assembly
    has_provenance_warning = False
    if os.path.exists(asm_file):
        with open(asm_file, 'r') as f:
            asm_content = f.read()
            if "WARNING: Assuming unknown pointer points to global memory" in asm_content:
                has_provenance_warning = True
    
    # Assemble to object
    ret, stdout, stderr = run_command(f"{RASM} assemble {asm_file} -o {pobj_file} --bank-size {BANK_SIZE} --max-immediate {MAX_IMMEDIATE}")
    if ret != 0:
        return False, f"Assembly failed: {stderr}", has_provenance_warning
    
    # Always link with at least crt0 for stack setup
    if use_full_runtime:
        # Link with full runtime (crt0 + libruntime.par + user code)
        link_cmd = f"{RLINK} {CRT0} {RUNTIME_LIB} {pobj_file} -f macro --standalone -o {bf_file}"
    else:
        # Link with crt0 only (provides stack setup but no runtime functions)
        link_cmd = f"{RLINK} {CRT0} {pobj_file} -f macro --standalone -o {bf_file}"
    
    ret, stdout, stderr = run_command(link_cmd)
    
    if ret != 0:
        return False, f"Linking failed: {stderr}", has_provenance_warning
    
    # Expand and run
    expanded_file = f"{BUILD_DIR}/{basename}_expanded.bfm"
    ret, stdout, stderr = run_command(f"bfm expand {bf_file} -o {expanded_file}")
    if ret != 0:
        return False, f"Macro expansion failed: {stderr}", has_provenance_warning
    
    ret, stdout, stderr = run_command(f"bf {expanded_file}", timeout=30)
    if ret == -1:
        return False, "Execution timeout", has_provenance_warning
    
    if expected_output and stdout != expected_output:
        if has_provenance_warning:
            return False, f"Output mismatch (likely due to pointer provenance issue). Expected: {repr(expected_output)}, Got: {repr(stdout)}", has_provenance_warning
        else:
            return False, f"Output mismatch. Expected: {repr(expected_output)}, Got: {repr(stdout)}", has_provenance_warning
    
    return True, "OK", has_provenance_warning

def cleanup_files():
    """Remove all generated files in the build directory"""
    # Patterns for files to clean up in build directory
    patterns = [
        f"{BUILD_DIR}/*.asm",
        f"{BUILD_DIR}/*.pobj", 
        f"{BUILD_DIR}/*.bf",
        f"{BUILD_DIR}/*_expanded.bfm"
    ]
    
    total_removed = 0
    for pattern in patterns:
        files = glob.glob(pattern)
        for file in files:
            try:
                os.remove(file)
                total_removed += 1
            except:
                pass  # Ignore errors during cleanup
    
    # Also clean up any stray files in current directory (for backwards compatibility)
    old_patterns = ["*.asm", "*.pobj", "*.bf", "*_expanded.bfm"]
    for pattern in old_patterns:
        files = glob.glob(pattern)
        for file in files:
            try:
                os.remove(file)
                total_removed += 1
            except:
                pass
    
    return total_removed

def main():
    # Check for command line flags
    no_cleanup = "--no-cleanup" in sys.argv
    clean_only = "--clean" in sys.argv
    
    # If --clean flag is provided, just clean and exit
    if clean_only:
        print("Cleaning build directory...")
        num_cleaned = cleanup_files()
        print(f"Removed {num_cleaned} files")
        return 0
    
    # Ensure runtime is built
    print("Building runtime library...")
    ret, _, stderr = run_command(f"cd {RUNTIME_DIR} && make clean && make all")
    if ret != 0:
        print(f"Failed to build runtime: {stderr}")
        return 1
    
    # Also ensure crt0.pobj is built (it's not part of the library archive)
    ret, _, stderr = run_command(f"cd {RUNTIME_DIR} && make crt0.pobj")
    if ret != 0:
        print(f"Failed to build crt0.pobj: {stderr}")
        return 1

    # Ensure build directory exists
    os.makedirs(BUILD_DIR, exist_ok=True)
    
    tests = [
        # Tests with crt0 only (stack setup but no runtime library functions)
        ("test_if_else_simple.c", "1:T\n2:F\n3:T\n4:F\n5:A\n6:2\n7:OK\n8:Y\n9:T\nA:T\nB:F\n", False),
        ("test_loops.c", "W:012\nF:ABC\nD:XYZ\nN:00 01 10 11 \nB:01\nC:0134\n", False),
        ("test_sizeof_simple.c", "1 2 2 :\n", False),
        ("test_globals.c", "*A\n", False),
        ("test_strings.c", "Plea", False),
        ("test_m3_comprehensive.c", "M3: OK!\nABC\nGood!", False),
        ("test_add.c", "Y\n", False),
        ("test_array_decl.c", "123\n", False),
        ("test_while_simple.c", "YY\n", False),
        ("test_hello.c", "Hello\n", False),
        ("test_simple_putchar.c", "AB\n", False),
        ("test_address_of.c", "OK\n", False),
        ("test_struct_basic.c", "Y\n", False),
        ("test_array_init.c", "1234\n", False),
        ("test_sizeof_verify.c", "1:Y\n2:Y\n3:Y\n", False),
        ("test_pointers_comprehensive.c", "12345\n", False),
        ("test_while_debug.c", "ABL0L1L2C\n", False),
        ("test_sizeof.c", "123456\n", False),
        ("test_sizeof_final.c", "YYYYYYYYY\n", False),
        ("test_strings_addr.c", "A", False),
        ("test_if_else.c", "1:T 2:F 3:T 4:F 5:A 6:2 7:T 8:T 9:T Y\n", False),
        ("test_struct_inline.c", "YY\n", False),
        ("test_struct_simple.c", "12345\n", False),
        ("test_puts_debug.c", "ABC\n", False),
        ("test_puts_string_literal.c", "XYZ\n", False),
        ("test-cond.c", "T", False),
        ("test_pointer_gritty.c", "7", False),
        ("test_inline_asm.c", "Y\n", False),
        
        # Tests with full runtime (crt0 + libruntime.par)
        ("test_runtime_simple.c", "RT\n", True),
        ("test_runtime_full.c", "Hi!\nOK\n", True),
        ("test_external_putchar.c", "Hi\n", True),
    ]
    
    # Sort tests alphabetically by filename
    tests.sort(key=lambda x: x[0])
    
    passed = 0
    failed = 0
    
    print("\nRunning tests...")
    print("-" * 60)
    
    for test_file, expected, use_full_runtime in tests:
        if not os.path.exists(test_file):
            print(f"SKIP {test_file}: File not found")
            continue
            
        success, message, has_provenance_warning = compile_and_run(test_file, expected, use_full_runtime)
        
        if success:
            if has_provenance_warning:
                print(f"{YELLOW}✓ {test_file}: PASSED WITH WARNINGS{NC} (pointer provenance unknown)")
            else:
                print(f"{GREEN}✓ {test_file}{NC}")
            passed += 1
        else:
            print(f"{RED}✗ {test_file}{NC}: {message}")
            failed += 1
    
    # Known failures section (tests that are expected to fail)
    known_failures = [
        "test_typedef.c",  # Typedef support not implemented
        "test_typedef_simple.c",  # Typedef support not implemented
        "test_struct_simple2.c",  # Uses typedef struct
        "test_struct_inline_simple.c",  # Inline struct definitions not supported
        "test_puts.c",  # Complex puts with loops
        "test_puts_simple.c",  # Stack pointer provenance issue
        "test_puts_string.c",  # Uses while loops
        "test_puts_global.c",  # Global array initializers not implemented
        "test_strings_simple.c",  # Function redefinition error
    ]
    
    # Sort known failures alphabetically
    known_failures.sort()
    
    print("\nRunning known failure tests...")
    print("-" * 60)
    
    known_failures_count = 0
    for test_file in known_failures:
        if not os.path.exists(test_file):
            print(f"SKIP {test_file}: File not found")
            continue
        
        basename = Path(test_file).stem
        asm_file = f"{BUILD_DIR}/{basename}.asm"
        
        # Try to compile it
        ret, stdout, stderr = run_command(f"{RCC} compile {test_file} -o {asm_file}")
        
        if ret != 0:
            print(f"{YELLOW}✓ {test_file}: EXPECTED FAIL{NC}")
            known_failures_count += 1
        else:
            # Check if it has provenance warnings
            has_warning = False
            if os.path.exists(asm_file):
                with open(asm_file, 'r') as f:
                    if "WARNING: Assuming unknown pointer points to global memory" in f.read():
                        has_warning = True
            
            if has_warning:
                print(f"{YELLOW}✓ {test_file}: COMPILES WITH WARNINGS{NC} (pointer provenance issue)")
                known_failures_count += 1
            else:
                print(f"{RED}✗ {test_file}: UNEXPECTED PASS{NC} (should have failed)")
    
    print("\n" + "="*60)
    print("              Test Results                ")
    print("="*60)
    print(f"Total:   {passed + failed}")
    print(f"Passed:  {GREEN}{passed}{NC}")
    print(f"Failed:  {RED}{failed}{NC}")
    if known_failures_count > 0:
        print(f"Known failures: {YELLOW}{known_failures_count}{NC}")
    
    # Determine exit code before cleanup
    if failed == 0:
        print(f"\n{GREEN}All tests passed!{NC}")
        exit_code = 0
    else:
        print(f"\n{YELLOW}Some tests failed. See above for details.{NC}")
        exit_code = 1
    
    # Clean up generated files unless --no-cleanup was specified
    if not no_cleanup:
        print("\nCleaning up generated files...")
        num_cleaned = cleanup_files()
        print(f"Removed {num_cleaned} files\n")
    else:
        print("\nSkipping cleanup (--no-cleanup specified)")
    
    return exit_code

if __name__ == "__main__":
    sys.exit(main())