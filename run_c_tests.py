#!/usr/bin/env python3

"""
Ripple C99 Compiler Test Suite
This script compiles and runs all C test files in the c-code directory
"""

import subprocess
import sys
import os
from pathlib import Path

# Colors for output
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
NC = '\033[0m'  # No Color

# Paths
RCC = "./target/release/rcc"
RBT = "rbt"
TEST_DIR = "c-code"

# Counters
total = 0
passed = 0
failed = 0
skipped = 0

# Test definitions: filename -> expected output
TESTS = {
    # Working tests with known output
    "test_if_else_simple.c": """1:T
2:F
3:T
4:F
5:A
6:2
7:OK
8:Y
9:T
A:T
B:F""",
    
    "test_loops.c": """W:012
F:ABC
D:XYZ
N:00 01 10 11 
B:01
C:0134""",
    
    "test_sizeof_simple.c": """1 2 2 :""",
    
    "test_globals.c": "*A\n",
    
    "test_strings.c": "Plea",
    
    "test_m3_comprehensive.c": """M3: OK!
ABC
Good!""",
    
    # New tests with output - Y/N pattern for verification
    "test_add.c": "Y\n",  # Tests if 5+10=15
    "test_array_decl.c": "123\n",  # Tests array indexing
    "test_while_simple.c": "YY\n",  # Tests loop iterations and final value
    "test_hello.c": "Hello\n",
    "test_simple_putchar.c": "AB\n",
    "test_address_of.c": "OK\n",
    "test_struct_basic.c": "Y\n",  # Tests if struct member sum = 30
    "test_array_init.c": "1234\n",  # Tests array initializers with {}
    "test_sizeof_verify.c": "1:Y\n2:Y\n3:Y\n",
    "test_pointers_comprehensive.c": "12345\n",
    "test_while_debug.c": "ABL0L1L2C\n",
    "test_sizeof.c": "123456\n",  # Fixed to not use recursion
    "test_sizeof_final.c": "YYYYYYYYY\n",
    "test_strings_addr.c": "A\n",
    "test_if_else.c": "1:T 2:F 3:T 4:F 5:A 6:2 7:T 8:T 9:T Y\n",  # Fixed pre-increment
    "test_struct_inline.c": "YY\n",  # Tests inline struct members
    "test_struct_simple.c": "12345\n",  # Tests standalone struct type definitions
    "test_puts_debug.c": "ABC\n",  # Tests consecutive array indexing with global strings
    "test_puts_string_literal.c": "XYZ\n",  # Tests puts with string literals
    "test-cond.c": "T\n",  # Simple conditional test
    "test_pointer_gritty.c": "7\n",  # Tests passing address-of local variable to function
    "test_inline_asm.c": "Y\n",  # Tests inline assembly support
    # Note: test_puts.c partially works but has issues with stack arrays due to pointer provenance
}

# Tests that should compile but may not run correctly yet
COMPILE_ONLY = [
    # Currently empty - all tests have been fixed!
]

# Tests that currently fail to compile (known issues)
KNOWN_FAILURES = [
    # Typedef support - partially implemented but needs parser symbol table
    "test_typedef.c",  # Parser needs to track typedef names during parsing (classic C parsing problem)
    "test_typedef_simple.c",  # Same typedef parsing issue
    "test_struct_simple2.c",  # Uses typedef struct - blocked by typedef parsing
    
    # Other struct issues
    "test_struct_inline_simple.c",  # Inline struct definitions not supported
    
    # Pointer provenance issues (deferred to M4)
    "test_puts.c",  # Complex puts with loops - also needs while loop support
    "test_puts_simple.c",  # Stack pointer provenance issue - loads from wrong memory bank
    "test_puts_string.c",  # Uses while loops which aren't fully implemented
    "test_puts_global.c",  # Global array initializers not yet implemented (deferred to M4)
    
    # Misc issues
    "test_strings_simple.c",  # Function redefinition error (puts declared twice)
]

def run_command(cmd, capture_output=True):
    """Run a command and return success status and output"""
    try:
        if capture_output:
            result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
            return result.returncode == 0, result.stdout
        else:
            result = subprocess.run(cmd, shell=True)
            return result.returncode == 0, ""
    except Exception as e:
        return False, str(e)

def run_test(test_file, expected_output=None):
    """Run a single test"""
    global total, passed, failed, skipped
    
    test_name = Path(test_file).stem
    total += 1
    
    print(f"Testing {test_name:30} ... ", end="")
    
    # Check if file exists
    test_path = Path(TEST_DIR) / test_file
    if not test_path.exists():
        print(f"{YELLOW}SKIPPED{NC} (file not found)")
        skipped += 1
        return
    
    # Compile the test
    asm_path = Path(TEST_DIR) / f"{test_name}.asm"
    success, _ = run_command(f"{RCC} compile {test_path} -o {asm_path}")
    
    if not success:
        print(f"{RED}FAILED{NC} (compilation error)")
        failed += 1
        return
    
    # Check for provenance warnings in the generated assembly
    has_provenance_warning = False
    if asm_path.exists():
        with open(asm_path, 'r') as f:
            asm_content = f.read()
            if "WARNING: Assuming unknown pointer points to global memory" in asm_content:
                has_provenance_warning = True
    
    # Run the test if we have expected output
    if expected_output:
        success, actual_output = run_command(f"{RBT} {asm_path} --run")
        actual_output = actual_output.strip()
        
        # Compare output (strip trailing whitespace for comparison)
        expected_stripped = expected_output.strip()
        if actual_output == expected_stripped:
            if has_provenance_warning:
                print(f"{YELLOW}PASSED WITH WARNINGS{NC} (pointer provenance unknown)")
                passed += 1  # Still count as passed but with warning
            else:
                print(f"{GREEN}PASSED{NC}")
                passed += 1
        else:
            if has_provenance_warning:
                print(f"{RED}FAILED{NC} (likely due to pointer provenance issue)")
            else:
                print(f"{RED}FAILED{NC}")
            expected_first = expected_stripped.split('\n')[0] if expected_stripped else ""
            actual_first = actual_output.split('\n')[0] if actual_output else ""
            print(f"  Expected: {repr(expected_first)}...")
            print(f"  Got:      {repr(actual_first)}...")
            failed += 1
    else:
        # Just check compilation
        if has_provenance_warning:
            print(f"{YELLOW}COMPILED WITH WARNINGS{NC} (pointer provenance unknown)")
        else:
            print(f"{GREEN}COMPILED{NC}")
        passed += 1

def cleanup_asm_files():
    """Remove all generated .asm files in the test directory"""
    import glob
    asm_files = glob.glob(f"{TEST_DIR}/*.asm")
    for asm_file in asm_files:
        try:
            os.remove(asm_file)
        except:
            pass  # Ignore errors during cleanup
    return len(asm_files)

def main():
    # Check for --no-cleanup flag
    no_cleanup = "--no-cleanup" in sys.argv
    
    print("Building the compiler...")
    success, _ = run_command("cargo build --release")
    if not success:
        print(f"{RED}Failed to build compiler{NC}")
        return 1
    
    print("\n" + "="*42)
    print("     Ripple C99 Compiler Test Suite      ")
    print("="*42 + "\n")
    
    print("Running tests with expected output...\n")
    
    # Run tests with expected output
    for test_file, expected_output in TESTS.items():
        run_test(test_file, expected_output)
    
    print("\nRunning compile-only tests...\n")
    
    # Run compile-only tests
    for test_file in COMPILE_ONLY:
        run_test(test_file)
    
    print("\nRunning known failure tests...\n")
    
    # Run known failure tests (we expect these to fail)
    known_failures_count = 0
    for test_file in KNOWN_FAILURES:
        test_name = Path(test_file).stem
        test_path = Path(TEST_DIR) / test_file
        if test_path.exists():
            print(f"Testing {test_name:30} ... ", end="")
            asm_path = Path(TEST_DIR) / f"{test_name}.asm"
            success, _ = run_command(f"{RCC} compile {test_path} -o {asm_path}")
            if not success:
                print(f"{YELLOW}EXPECTED FAIL{NC}")
                known_failures_count += 1
            else:
                # Check if it has provenance warnings
                has_warning = False
                if asm_path.exists():
                    with open(asm_path, 'r') as f:
                        if "WARNING: Assuming unknown pointer points to global memory" in f.read():
                            has_warning = True
                if has_warning:
                    print(f"{YELLOW}COMPILES WITH WARNINGS{NC} (pointer provenance issue)")
                else:
                    print(f"{RED}UNEXPECTED PASS{NC} (should have failed)")
    
    # Print results
    print("\n" + "="*42)
    print("              Test Results                ")
    print("="*42 + "\n")
    
    print(f"Total:   {total}")
    print(f"Passed:  {GREEN}{passed}{NC}")
    print(f"Failed:  {RED}{failed}{NC}")
    print(f"Skipped: {YELLOW}{skipped}{NC}")
    if known_failures_count > 0:
        print(f"Known failures: {YELLOW}{known_failures_count}{NC}")
    print()
    
    # Determine exit code before cleanup
    if failed == 0 and skipped == 0:
        print(f"{GREEN}All tests passed!{NC}")
        exit_code = 0
    elif failed == 0:
        print(f"{YELLOW}All run tests passed, but some were skipped.{NC}")
        exit_code = 0
    else:
        print(f"{RED}Some tests failed!{NC}")
        exit_code = 1
    
    # Clean up generated .asm files unless --no-cleanup was specified
    if not no_cleanup:
        print("\nCleaning up generated .asm files...")
        num_cleaned = cleanup_asm_files()
        print(f"Removed {num_cleaned} .asm files\n")
    else:
        print("\nSkipping cleanup (--no-cleanup specified)")
    
    return exit_code

if __name__ == "__main__":
    sys.exit(main())