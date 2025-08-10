#!/usr/bin/env python3
"""
Test runner for the Ripple C compiler

Can be run from either the project root or the c-test directory.

Usage: python3 c-test/run_tests.py [options] [test_name]
   or: python3 run_tests.py [options] [test_name] (from c-test dir)

Options:
  --no-cleanup  Don't clean up generated files after tests
  --clean       Only clean the build directory and exit
  --timeout N   Set timeout in seconds for test execution (default: 2)
  --verbose     Show output from test programs as they run
  test_name     Optional: Name of a single test to run (without path or .c extension)

Examples:
  python3 c-test/run_tests.py                    # Run all tests (from root)
  python3 c-test/run_tests.py test_hello         # Run single test by name
  python3 c-test/run_tests.py test_add.c         # Also accepts with .c extension
  python3 c-test/run_tests.py --no-cleanup test_add  # Run single test, keep artifacts
  python3 c-test/run_tests.py --timeout 10       # Run with 10 second timeout
  python3 c-test/run_tests.py --verbose          # Show test program output

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

# Detect if script is being run from root or c-test directory
# We check if we're in a directory containing c-test or if we're already in c-test
current_dir = os.getcwd()
if os.path.exists('c-test') and os.path.isdir('c-test'):
    # Running from root directory
    BASE_DIR = "c-test"
    PROJECT_ROOT = "."
elif os.path.basename(current_dir) == 'c-test':
    # Running from c-test directory
    BASE_DIR = "."
    PROJECT_ROOT = ".."
else:
    # Try to detect based on parent directory
    parent_dir = os.path.dirname(current_dir)
    if os.path.exists(os.path.join(parent_dir, 'c-test')):
        BASE_DIR = os.path.join(parent_dir, 'c-test')
        PROJECT_ROOT = parent_dir
    else:
        # Default to assuming we're in c-test
        BASE_DIR = "."
        PROJECT_ROOT = ".."

# Build paths (relative to project root)
RCC = f"{PROJECT_ROOT}/target/release/rcc"
RASM = f"{PROJECT_ROOT}/src/ripple-asm/target/release/rasm"
RLINK = f"{PROJECT_ROOT}/src/ripple-asm/target/release/rlink"
RBT = "rbt"  # Global binary

# Runtime files
RUNTIME_DIR = f"{PROJECT_ROOT}/runtime"
CRT0 = f"{RUNTIME_DIR}/crt0.pobj"
RUNTIME_LIB = f"{RUNTIME_DIR}/libruntime.par"

# Build directory for temporary artifacts (inside c-test)
BUILD_DIR = f"{BASE_DIR}/build"

# Assembler settings (must match Makefile)
BANK_SIZE = 4096
MAX_IMMEDIATE = 65535

def run_command(cmd, timeout=2):
    """Run a command with timeout"""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout)
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired as e:
        # Capture any partial output that was produced before timeout
        stdout = e.stdout if e.stdout else ""
        stderr = e.stderr if e.stderr else ""
        timeout_msg = f"Timeout after {timeout}s"
        if stderr:
            timeout_msg = f"{timeout_msg}\nStderr: {stderr}"
        return -1, stdout, timeout_msg

def compile_and_run(c_file, expected_output, use_full_runtime=True, timeout=2, verbose=False):
    """Compile a C file and run it, checking output
    
    Args:
        c_file: The C source file to compile
        expected_output: Expected output from the program
        use_full_runtime: If True, link with full runtime (crt0 + libruntime.par)
                         If False, link with crt0 only (for stack setup)
        timeout: Timeout in seconds for execution
        verbose: If True, show program output
    
    Returns:
        tuple: (success, message, has_provenance_warning)
    """
    basename = Path(c_file).stem
    asm_file = f"{BUILD_DIR}/{basename}.asm"
    ir_file = f"{BUILD_DIR}/{basename}.ir"
    pobj_file = f"{BUILD_DIR}/{basename}.pobj"
    bf_file = f"{BUILD_DIR}/{basename}.bfm"
    
    # Clean up previous files
    for f in [asm_file, ir_file, pobj_file, bf_file, f"{BUILD_DIR}/{basename}_expanded.bf"]:
        if os.path.exists(f):
            os.remove(f)
    
    # Compile C to assembly (and save IR to build directory)
    ret, stdout, stderr = run_command(f"{RCC} compile {c_file} -o {asm_file} --save-ir --ir-output {ir_file}")
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
    expanded_file = f"{BUILD_DIR}/{basename}_expanded.bf"
    ret, stdout, stderr = run_command(f"bfm expand {bf_file} -o {expanded_file}")
    if ret != 0:
        return False, f"Macro expansion failed: {stderr}", has_provenance_warning
    
    ret, stdout, stderr = run_command(f"bf {expanded_file} --cell-size 16 --tape-size 150000000", timeout=timeout)
    if ret == -1:
        # Show partial output if verbose mode is enabled and timeout occurred
        if verbose and stdout:
            print(f"    Partial output before timeout: {repr(stdout)}")
        return False, stderr, has_provenance_warning
    
    if verbose and stdout:
        print(f"    Output: {repr(stdout)}")
    
    if expected_output is None:
        # No expected output specified, just return the actual output
        return True, stdout, has_provenance_warning
    elif stdout != expected_output:
        if has_provenance_warning:
            return False, f"Output mismatch (likely due to pointer provenance issue). Expected: {repr(expected_output)}, Got: {repr(stdout)}", has_provenance_warning
        else:
            return False, f"Output mismatch. Expected: {repr(expected_output)}, Got: {repr(stdout)}", has_provenance_warning
    
    return True, "OK", has_provenance_warning

def cleanup_test_files(basename):
    """Remove generated files for a specific test"""
    files_to_remove = [
        f"{BUILD_DIR}/{basename}.asm",
        f"{BUILD_DIR}/{basename}.ir",
        f"{BUILD_DIR}/{basename}.pobj",
        f"{BUILD_DIR}/{basename}.bfm",
        f"{BUILD_DIR}/{basename}_expanded.bf"
    ]
    
    removed = 0
    for file in files_to_remove:
        if os.path.exists(file):
            try:
                os.remove(file)
                removed += 1
            except:
                pass
    
    return removed

def cleanup_files():
    """Remove all generated files in the build directory"""
    # Patterns for files to clean up in build directory
    patterns = [
        f"{BUILD_DIR}/*.asm",
        f"{BUILD_DIR}/*.ir",
        f"{BUILD_DIR}/*.pobj", 
        f"{BUILD_DIR}/*.bfm",
        f"{BUILD_DIR}/*_expanded.bf"
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
    old_patterns = ["*.asm", "*.pobj", "*.bfm", "*_expanded.bf"]
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
    verbose = "--verbose" in sys.argv
    single_test = None
    timeout = 10  # Default timeout in seconds
    
    # Parse arguments
    i = 1
    while i < len(sys.argv):
        arg = sys.argv[i]
        if arg == "--timeout":
            if i + 1 < len(sys.argv):
                try:
                    timeout = int(sys.argv[i + 1])
                    i += 1  # Skip the timeout value
                except ValueError:
                    print(f"Error: Invalid timeout value '{sys.argv[i + 1]}'")
                    return 1
            else:
                print("Error: --timeout requires a value")
                return 1
        elif not arg.startswith("--"):
            # Accept test names with or without .c extension
            single_test = arg
        i += 1
    
    # If --clean flag is provided, just clean and exit
    if clean_only:
        print("Cleaning build directory...")
        num_cleaned = cleanup_files()
        print(f"Removed {num_cleaned} files")
        return 0
    
    # Ensure runtime is built
    verbose_msg = ", verbose" if verbose else ""
    print(f"Building runtime library (timeout: {timeout}s{verbose_msg})...")
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
        (f"{BASE_DIR}/tests/test_inline_asm.c", "Y\n", False),
        
        # Tests with full runtime (crt0 + libruntime.par)
        (f"{BASE_DIR}/tests-runtime/test_runtime_simple.c", "RT\n", True),
        (f"{BASE_DIR}/tests-runtime/test_runtime_full.c", "Hi!\nOK\n", True),
        (f"{BASE_DIR}/tests-runtime/test_external_putchar.c", "Hi\n", True),
        
        # Tests moved from tests/ directory (all use putchar)
        (f"{BASE_DIR}/tests-runtime/test_if_else_simple.c", "1:T\n2:F\n3:T\n4:F\n5:A\n6:2\n7:OK\n8:Y\n9:T\nA:T\nB:F\n", True),
        (f"{BASE_DIR}/tests-runtime/test_loops.c", "W:012\nF:ABC\nD:XYZ\nN:00 01 10 11 \nB:01\nC:0134\n", True),
        (f"{BASE_DIR}/tests-runtime/test_sizeof_simple.c", "1 2 2 :\n", True),
        (f"{BASE_DIR}/tests-runtime/test_globals.c", "*A\n", True),
        (f"{BASE_DIR}/tests-runtime/test_strings.c", "Plea", True),
        (f"{BASE_DIR}/tests-runtime/test_m3_comprehensive.c", "M3: OK!\nABC\nGood!\n", True),
        (f"{BASE_DIR}/tests-runtime/test_add.c", "Y\n", True),
        (f"{BASE_DIR}/tests-runtime/test_array_decl.c", "123\n", True),
        (f"{BASE_DIR}/tests-runtime/test_while_simple.c", "YY\n", True),
        (f"{BASE_DIR}/tests-runtime/test_hello.c", "Hello\n", True),
        (f"{BASE_DIR}/tests-runtime/test_simple_putchar.c", "AB\n", True),
        (f"{BASE_DIR}/tests-runtime/test_address_of.c", "OK\n", True),
        (f"{BASE_DIR}/tests-runtime/test_struct_basic.c", "Y\n", True),
        (f"{BASE_DIR}/tests-runtime/test_array_init.c", "1234\n", True),
        (f"{BASE_DIR}/tests-runtime/test_sizeof_verify.c", "1:Y\n2:Y\n3:Y\n", True),
        (f"{BASE_DIR}/tests-runtime/test_pointers_comprehensive.c", "12345\n", True),
        (f"{BASE_DIR}/tests-runtime/test_while_debug.c", "ABL0L1L2C\n", True),
        (f"{BASE_DIR}/tests-runtime/test_sizeof.c", "123456\n", True),
        (f"{BASE_DIR}/tests-runtime/test_sizeof_final.c", "YYYYYYYYY\n", True),
        (f"{BASE_DIR}/tests-runtime/test_strings_addr.c", "A", True),
        (f"{BASE_DIR}/tests-runtime/test_if_else.c", "1:T 2:F 3:T 4:F 5:A 6:2 7:T 8:T 9:T Y\n", True),

        (f"{BASE_DIR}/tests-runtime/test_puts.c", "Hello, World!\n", True),


        # Pointer provenance tests with fat pointers
        (f"{BASE_DIR}/tests/test_pointer_provenance.c", "GSSSS\n", True),
        (f"{BASE_DIR}/tests/test_pointer_phi.c", "1234\n", True),
        (f"{BASE_DIR}/tests-runtime/test_struct_inline.c", "YY\n", True),
        (f"{BASE_DIR}/tests-runtime/test_struct_simple.c", "12345\n", True),
        (f"{BASE_DIR}/tests-runtime/test_puts_debug.c", "ABC\n", True),
        (f"{BASE_DIR}/tests-runtime/test_puts_string_literal.c", "XYZ\n", True),
        (f"{BASE_DIR}/tests-runtime/test-cond.c", "T", True),
        (f"{BASE_DIR}/tests-runtime/test_pointer_gritty.c", "7", True),
        (f"{BASE_DIR}/tests-runtime/test_bool.c", "12\n", True),
        (f"{BASE_DIR}/tests-runtime/test_complex_bool.c", "12345\n", True),
        (f"{BASE_DIR}/tests-runtime/test_complex_simple.c", "123\n", True),
        (f"{BASE_DIR}/tests-runtime/test_mul.c", "Y", True),
    ]
    
    # If single test specified, filter the test list
    if single_test:
        # Normalize the test name - remove .c extension if present, remove path if present
        test_name = os.path.basename(single_test)
        if test_name.endswith('.c'):
            test_name = test_name[:-2]
        
        # Find the test in the list by matching the basename without extension
        matching_test = None
        for test in tests:
            test_file_basename = os.path.basename(test[0])
            if test_file_basename.endswith('.c'):
                test_file_basename = test_file_basename[:-2]
            if test_file_basename == test_name:
                matching_test = test
                break
        
        if not matching_test:
            # If not found in tests list, search for the file in test directories
            possible_paths = [
                f"{BASE_DIR}/tests/{test_name}.c",
                f"{BASE_DIR}/tests-runtime/{test_name}.c",
                f"{BASE_DIR}/tests-known-failures/{test_name}.c",
            ]
            
            found_path = None
            for path in possible_paths:
                if os.path.exists(path):
                    found_path = path
                    break
            
            if not found_path:
                print(f"Error: Test '{test_name}' not found in any test directory")
                print(f"Searched in: tests/, tests-runtime/, tests-known-failures/")
                return 1
            
            # Determine if it should use runtime based on directory
            use_runtime = "tests-runtime" in found_path or "tests-known-failures" in found_path
            
            print(f"Running single test: {found_path}")
            print(f"Note: Expected output not defined in test list, will show actual output")
            print("-" * 60)
            
            success, message, has_provenance_warning = compile_and_run(found_path, None, use_runtime, timeout, verbose)
            
            if success:
                if has_provenance_warning:
                    print(f"{YELLOW}✓ {found_path}: COMPILED WITH WARNINGS{NC} (pointer provenance unknown)")
                else:
                    print(f"{GREEN}✓ {found_path}: COMPILED AND RAN{NC}")
                print(f"Output: {repr(message)}")
            else:
                print(f"{RED}✗ {found_path}{NC}: {message}")
            
            if not no_cleanup:
                cleanup_files()
            
            return 0 if success else 1
        
        # Found in test list, run with expected output
        tests = [matching_test]
        print(f"Running single test: {matching_test[0]}")
    
    # Sort tests alphabetically by filename
    tests.sort(key=lambda x: x[0])
    
    # Separate tests by runtime usage
    tests_without_runtime = [(f, e, r) for f, e, r in tests if not r]
    tests_with_runtime = [(f, e, r) for f, e, r in tests if r]
    
    passed = 0
    failed = 0
    
    # Run tests without runtime first (if any)
    if tests_without_runtime:
        print("\nTests without runtime (crt0 only):")
        print("-" * 60)
        
        for test_file, expected, use_full_runtime in tests_without_runtime:
            if not os.path.exists(test_file):
                print(f"SKIP {test_file}: File not found")
                continue
                
            success, message, has_provenance_warning = compile_and_run(test_file, expected, use_full_runtime, timeout, verbose)
            
            if success:
                if has_provenance_warning:
                    print(f"{YELLOW}✓ {test_file}: PASSED WITH WARNINGS{NC} (pointer provenance unknown)")
                else:
                    print(f"{GREEN}✓ {test_file}{NC}")
                passed += 1
            else:
                print(f"{RED}✗ {test_file}{NC}: {message}")
                failed += 1
    
    # Run tests with runtime
    if tests_with_runtime:
        print("\nTests with runtime (crt0 + libruntime):")
        print("-" * 60)
        
        for test_file, expected, use_full_runtime in tests_with_runtime:
            if not os.path.exists(test_file):
                print(f"SKIP {test_file}: File not found")
                continue
                
            success, message, has_provenance_warning = compile_and_run(test_file, expected, use_full_runtime, timeout, verbose)
            
            if success:
                if has_provenance_warning:
                    print(f"{YELLOW}✓ {test_file}: PASSED WITH WARNINGS{NC} (pointer provenance unknown)")
                else:
                    print(f"{GREEN}✓ {test_file}{NC}")
                passed += 1
            else:
                print(f"{RED}✗ {test_file}{NC}: {message}")
                failed += 1
    
    # Skip known failures section if running single test
    if single_test:
        if not no_cleanup:
            num_cleaned = cleanup_files()
            print(f"\nCleaned up {num_cleaned} generated files")
        
        return 0 if passed == 1 else 1
    
    # Known failures section (tests that are expected to fail)
    known_failures = [
        f"{BASE_DIR}/tests-known-failures/test_typedef.c",  # Typedef support not implemented
        f"{BASE_DIR}/tests-known-failures/test_typedef_simple.c",  # Typedef support not implemented
        f"{BASE_DIR}/tests-known-failures/test_struct_simple2.c",  # Uses typedef struct
        f"{BASE_DIR}/tests-known-failures/test_struct_inline_simple.c",  # Inline struct definitions not supported
        f"{BASE_DIR}/tests-known-failures/test_puts_simple.c",  # Stack pointer provenance issue
        f"{BASE_DIR}/tests-known-failures/test_puts_string.c",  # Uses while loops
        f"{BASE_DIR}/tests-known-failures/test_puts_global.c",  # Global array initializers not implemented
        f"{BASE_DIR}/tests-known-failures/test_strings_simple.c",  # Function redefinition error
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
        ir_file = f"{BUILD_DIR}/{basename}.ir"
        
        # Try to compile it (save IR to build directory)
        ret, stdout, stderr = run_command(f"{RCC} compile {test_file} -o {asm_file} --save-ir --ir-output {ir_file}")
        
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