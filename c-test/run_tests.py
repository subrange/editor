#!/usr/bin/env python3
"""
Test runner for the Ripple C compiler

Can be run from either the project root or the c-test directory.

Usage: python3 c-test/run_tests.py [options] [test_name ...]
   or: python3 run_tests.py [options] [test_name ...] (from c-test dir)

Options:
  --no-cleanup  Don't clean up generated files after tests
  --clean       Only clean the build directory and exit
  --timeout N   Set timeout in seconds for test execution (default: 2)
  --verbose     Show output from test programs as they run
  --backend B   Execution backend: 'bf' (default) or 'rvm' for Ripple VM
  --run         Use -t flag when running with RVM (trace mode)
  --add         Add a test to tests.json (usage: --add test.c "expected output")
  test_name     Optional: Names of tests to run (without path or .c extension)
                Can specify multiple tests

Examples:
  python3 c-test/run_tests.py                    # Run all tests (from root)
  python3 c-test/run_tests.py test_hello         # Run single test by name
  python3 c-test/run_tests.py test_add.c         # Also accepts with .c extension
  python3 c-test/run_tests.py test_add test_hello test_array_index_gep  # Run multiple tests
  python3 c-test/run_tests.py --no-cleanup test_add  # Run single test, keep artifacts
  python3 c-test/run_tests.py --timeout 10       # Run with 10 second timeout
  python3 c-test/run_tests.py --verbose          # Show test program output
  python3 c-test/run_tests.py --backend rvm      # Run tests on Ripple VM instead of BF
  python3 c-test/run_tests.py test_minimal_insert --run  # Compile and run with TUI debugger
  python3 c-test/run_tests.py --add test_new.c "Hello\\n"  # Add a new test to tests.json

Prerequisites:
- Build the compiler: cargo build --release (in project root)
- Build the assembler: cargo build --release (in src/ripple-asm)
- Build the RVM: cargo build --release (in rvm directory)
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
import json
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
RVM = f"{PROJECT_ROOT}/target/release/rvm"
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
        # Use binary mode to avoid UTF-8 decoding errors
        result = subprocess.run(cmd, shell=True, capture_output=True, text=False, timeout=timeout)
        # Try to decode as UTF-8, but replace invalid characters
        stdout = result.stdout.decode('utf-8', errors='replace') if result.stdout else ""
        stderr = result.stderr.decode('utf-8', errors='replace') if result.stderr else ""
        return result.returncode, stdout, stderr
    except subprocess.TimeoutExpired as e:
        # Capture any partial output that was produced before timeout
        stdout = e.stdout.decode('utf-8', errors='replace') if e.stdout else ""
        stderr = e.stderr.decode('utf-8', errors='replace') if e.stderr else ""
        timeout_msg = f"Timeout after {timeout}s"
        if stderr:
            timeout_msg = f"{timeout_msg}\nStderr: {stderr}"
        return -1, stdout, timeout_msg

def compile_and_run(c_file, expected_output, use_full_runtime=True, timeout=2, verbose=False, backend='bf', run_mode=False):
    """Compile a C file and run it, checking output
    
    Args:
        c_file: The C source file to compile
        expected_output: Expected output from the program
        use_full_runtime: If True, link with full runtime (crt0 + libruntime.par)
                         If False, link with crt0 only (for stack setup)
        timeout: Timeout in seconds for execution
        verbose: If True, show program output
        backend: Execution backend - 'bf' for Brainfuck or 'rvm' for Ripple VM
        run_mode: If True, use -t flag when running with RVM
    
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
    
    # Link the program
    if backend == 'rvm':
        # For RVM, link to binary format
        bin_file = f"{BUILD_DIR}/{basename}.bin"
        if use_full_runtime:
            link_cmd = f"{RLINK} {CRT0} {RUNTIME_LIB} {pobj_file} -f binary -o {bin_file}"
        else:
            link_cmd = f"{RLINK} {CRT0} {pobj_file} -f binary -o {bin_file}"
        
        ret, stdout, stderr = run_command(link_cmd)
        if ret != 0:
            return False, f"Linking failed: {stderr}", has_provenance_warning
        
        # Run on RVM
        rvm_flags = "-t" if run_mode else ""
        ret, stdout, stderr = run_command(f"{RVM} {bin_file} --memory 4294967296 {rvm_flags}".strip(), timeout=timeout)
        if ret == -1:
            if verbose and stdout:
                print(f"    Partial output before timeout: {repr(stdout)}")
            return False, stderr, has_provenance_warning

        run_command(f"{RASM} disassemble {bin_file} -o {BUILD_DIR}/{basename}.disassembly.asm")  # Save disassembly for debugging
    else:
        # For BF backend, link to macro format
        if use_full_runtime:
            link_cmd = f"{RLINK} {CRT0} {RUNTIME_LIB} {pobj_file} -f macro --standalone -o {bf_file}"
        else:
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
        f"{BUILD_DIR}/{basename}.bin",
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
        f"{BUILD_DIR}/*.bin",
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
    run_mode = "--run" in sys.argv
    add_mode = "--add" in sys.argv
    test_files = []  # Changed from single_test to support multiple tests
    timeout = 2  # Default timeout in seconds
    backend = 'rvm'  # Default backend
    
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
        elif arg == "--backend":
            if i + 1 < len(sys.argv):
                backend_val = sys.argv[i + 1].lower()
                if backend_val in ['bf', 'rvm']:
                    backend = backend_val
                    i += 1  # Skip the backend value
                else:
                    print(f"Error: Invalid backend '{sys.argv[i + 1]}'. Must be 'bf' or 'rvm'")
                    return 1
            else:
                print("Error: --backend requires a value ('bf' or 'rvm')")
                return 1
        elif not arg.startswith("--"):
            # Accept test names with or without .c extension
            test_files.append(arg)
        i += 1
    
    # Handle --add mode to add a new test
    if add_mode:
        if len(test_files) < 1:
            print("Error: --add requires a test file path")
            print("Usage: python3 run_tests.py --add test.c \"expected output\"")
            return 1
        
        test_file = test_files[0]
        expected_output = test_files[1] if len(test_files) > 1 else None
        
        # Determine if it's a regular test or known failure
        is_known_failure = "known-failures" in test_file or expected_output is None
        
        # Normalize the path
        if not test_file.startswith("tests"):
            if is_known_failure:
                test_file = f"tests-known-failures/{os.path.basename(test_file)}"
            else:
                test_file = f"tests/{os.path.basename(test_file)}"
        
        # Load existing tests
        tests_json_path = f"{BASE_DIR}/tests.json"
        test_data = {"tests": [], "known_failures": []}
        
        if os.path.exists(tests_json_path):
            try:
                with open(tests_json_path, 'r') as f:
                    test_data = json.load(f)
            except:
                pass
        
        # Add the new test
        if is_known_failure:
            # Check if already exists
            existing = [t for t in test_data.get('known_failures', []) if t['file'] == test_file]
            if existing:
                print(f"Test {test_file} already exists in known failures")
            else:
                test_data.setdefault('known_failures', []).append({
                    "file": test_file,
                    "description": "Added via command line"
                })
                print(f"Added {test_file} to known failures")
        else:
            # Check if already exists
            existing = [t for t in test_data.get('tests', []) if t['file'] == test_file]
            if existing:
                print(f"Test {test_file} already exists, updating expected output")
                existing[0]['expected'] = expected_output
            else:
                test_data.setdefault('tests', []).append({
                    "file": test_file,
                    "expected": expected_output,
                    "use_runtime": True,
                    "description": "Added via command line"
                })
                print(f"Added {test_file} with expected output: {repr(expected_output)}")
        
        # Save the updated tests
        with open(tests_json_path, 'w') as f:
            json.dump(test_data, f, indent=2)
        
        print(f"Updated {tests_json_path}")
        return 0
    
    # If --clean flag is provided, just clean and exit
    if clean_only:
        print("Cleaning build directory...")
        num_cleaned = cleanup_files()
        print(f"Removed {num_cleaned} files")
        return 0
    
    # Ensure runtime is built
    backend_msg = f", backend: {backend.upper()}" if backend != 'bf' else ""
    verbose_msg = ", verbose" if verbose else ""
    print(f"Building runtime library (timeout: {timeout}s{backend_msg}{verbose_msg})...")
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
    
    # Load tests from JSON file
    tests_json_path = f"{BASE_DIR}/tests.json"
    tests = []
    known_failures = []
    
    if os.path.exists(tests_json_path):
        try:
            with open(tests_json_path, 'r') as f:
                test_data = json.load(f)
                
            # Load regular tests
            for test in test_data.get('tests', []):
                test_file = f"{BASE_DIR}/{test['file']}"
                expected = test.get('expected', None)
                use_runtime = test.get('use_runtime', True)
                tests.append((test_file, expected, use_runtime))
            
            # Load known failures
            for failure in test_data.get('known_failures', []):
                test_file = f"{BASE_DIR}/{failure['file']}"
                known_failures.append(test_file)
                
        except (json.JSONDecodeError, KeyError) as e:
            print(f"Warning: Failed to load tests from {tests_json_path}: {e}")
            print("Using empty test list")
    else:
        print(f"Warning: Test configuration file {tests_json_path} not found")
        print("Using empty test list")
    
    # Handle --run mode for interactive debugging
    if run_mode and test_files:
        # In run mode, we just compile and then run interactively
        print("Interactive run mode enabled (TUI debugger)")
        print("-" * 60)
        
        for test_file in test_files:
            # Normalize the test name
            test_name = os.path.basename(test_file)
            if test_name.endswith('.c'):
                test_name = test_name[:-2]
            
            # Find the test or construct the path
            test_path = None
            use_runtime = True
            
            # Check if it's in the known test list
            for test in tests:
                test_file_basename = os.path.basename(test[0])
                if test_file_basename.endswith('.c'):
                    test_file_basename = test_file_basename[:-2]
                if test_file_basename == test_name:
                    test_path = test[0]
                    use_runtime = test[2]
                    break
            
            # If not found, search for the file
            if not test_path:
                possible_paths = [
                    f"{BASE_DIR}/tests/{test_name}.c",
                    f"{BASE_DIR}/tests-known-failures/{test_name}.c",
                ]
                
                for path in possible_paths:
                    if os.path.exists(path):
                        test_path = path
                        break
            
            if not test_path:
                print(f"Error: Test '{test_name}' not found")
                continue
            
            print(f"\nCompiling: {test_path}")
            
            # Compile the test
            basename = Path(test_path).stem
            asm_file = f"{BUILD_DIR}/{basename}.asm"
            ir_file = f"{BUILD_DIR}/{basename}.ir"
            pobj_file = f"{BUILD_DIR}/{basename}.pobj"
            bin_file = f"{BUILD_DIR}/{basename}.bin"
            
            # Compile C to assembly
            ret, stdout, stderr = run_command(f"{RCC} compile {test_path} -o {asm_file} --save-ir --ir-output {ir_file}")
            if ret != 0:
                print(f"{RED}✗ Compilation failed{NC}: {stderr}")
                continue
            
            # Assemble to object
            ret, stdout, stderr = run_command(f"{RASM} assemble {asm_file} -o {pobj_file} --bank-size {BANK_SIZE} --max-immediate {MAX_IMMEDIATE}")
            if ret != 0:
                print(f"{RED}✗ Assembly failed{NC}: {stderr}")
                continue
            
            # Link to binary
            if use_runtime:
                link_cmd = f"{RLINK} {CRT0} {RUNTIME_LIB} {pobj_file} -f binary -o {bin_file}"
            else:
                link_cmd = f"{RLINK} {CRT0} {pobj_file} -f binary -o {bin_file}"
            
            ret, stdout, stderr = run_command(link_cmd)
            if ret != 0:
                print(f"{RED}✗ Linking failed{NC}: {stderr}")
                continue
            
            print(f"{GREEN}✓ Successfully built {bin_file}{NC}")
            print(f"Running: {RVM} {bin_file} -t")
            print("-" * 60)
            
            # Run interactively with TUI debugger
            import subprocess
            subprocess.run([RVM, bin_file, "-t"])
        
        return 0
    
    # If specific tests specified, filter the test list
    if test_files:
        tests_to_run = []
        unknown_tests = []
        
        for test_file in test_files:
            # Normalize the test name - remove .c extension if present, remove path if present
            test_name = os.path.basename(test_file)
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
            
            if matching_test:
                tests_to_run.append(matching_test)
            else:
                # If not found in tests list, search for the file in test directories
                possible_paths = [
                    f"{BASE_DIR}/tests/{test_name}.c",
                    f"{BASE_DIR}/tests-known-failures/{test_name}.c",
                ]
                
                found_path = None
                for path in possible_paths:
                    if os.path.exists(path):
                        found_path = path
                        break
                
                if not found_path:
                    print(f"Error: Test '{test_name}' not found in any test directory")
                    continue
                
                # Determine if it should use runtime based on directory
                use_runtime = "tests" in found_path or "tests-known-failures" in found_path
                unknown_tests.append((found_path, None, use_runtime))
        
        # Run known tests first
        if tests_to_run:
            tests = tests_to_run
            if len(tests_to_run) == 1:
                print(f"Running single test: {tests_to_run[0][0]}")
            else:
                print(f"Running {len(tests_to_run)} tests from test list")
        else:
            tests = []
        
        # Handle unknown tests separately
        if unknown_tests:
            if not tests_to_run:
                print(f"Running {len(unknown_tests)} test(s) not in test list")
            
            for found_path, _, use_runtime in unknown_tests:
                print(f"\nRunning test: {found_path}")
                print(f"Note: Expected output not defined in test list, will show actual output")
                print("-" * 60)
                
                success, message, has_provenance_warning = compile_and_run(found_path, None, use_runtime, timeout, verbose, backend, False)
                
                if success:
                    if has_provenance_warning:
                        print(f"{YELLOW}✓ {found_path}: COMPILED WITH WARNINGS{NC} (pointer provenance unknown)")
                    else:
                        print(f"{GREEN}✓ {found_path}: COMPILED AND RAN{NC}")
                    print(f"Output: {repr(message)}")
                else:
                    print(f"{RED}✗ {found_path}{NC}: {message}")
            
            if not tests_to_run:  # Only return early if we had no known tests
                if not no_cleanup:
                    cleanup_files()
                return 0
    
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
                
            success, message, has_provenance_warning = compile_and_run(test_file, expected, use_full_runtime, timeout, verbose, backend, run_mode)
            
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
                
            success, message, has_provenance_warning = compile_and_run(test_file, expected, use_full_runtime, timeout, verbose, backend, run_mode)
            
            if success:
                if has_provenance_warning:
                    print(f"{YELLOW}✓ {test_file}: PASSED WITH WARNINGS{NC} (pointer provenance unknown)")
                else:
                    print(f"{GREEN}✓ {test_file}{NC}")
                passed += 1
            else:
                print(f"{RED}✗ {test_file}{NC}: {message}")
                failed += 1
    
    # Skip known failures section if running specific tests
    if test_files:
        # if not no_cleanup:
        #     num_cleaned = cleanup_files()
        #     print(f"\nCleaned up {num_cleaned} generated files")
        
        return 0 if failed == 0 else 1
    
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
    # if not no_cleanup:
    #     print("\nCleaning up generated files...")
    #     num_cleaned = cleanup_files()
    #     print(f"Removed {num_cleaned} files\n")
    # else:
    #     print("\nSkipping cleanup (--no-cleanup specified)")
    
    return exit_code

if __name__ == "__main__":
    sys.exit(main())