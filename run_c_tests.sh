#!/bin/bash

# Ripple C99 Compiler Test Suite
# This script compiles and runs all C test files in the c-code directory

set -e  # Exit on first error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

# Paths
RCC="./target/release/rcc"
RBT="rbt"
TEST_DIR="c-code"

# Build the compiler if needed
echo "Building the compiler..."
cargo build --release 2>&1 | tail -1

echo ""
echo "=========================================="
echo "     Ripple C99 Compiler Test Suite      "
echo "=========================================="
echo ""

# Create a function to add tests
declare -A TESTS

# Define expected outputs for each test
TESTS["test_if_else_simple.c"]="1:T
2:F
3:T
4:F
5:A
6:2
7:OK
8:Y
9:T
A:T
B:F"

TESTS["test_loops.c"]="W:012
F:ABC
D:XYZ
N:00 01 10 11 
B:01
C:0134"

TESTS["test_sizeof_simple.c"]="sizeof(char) = 1
sizeof(int) = 2
sizeof(long) = 4
sizeof(int*) = 2"

TESTS["test_globals.c"]="Global: 42
Modified: 100"

TESTS["test_address_of.c"]="Value: 42
Pointer: 42
Modified: 100"

TESTS["test_strings.c"]="Hello, World!"

TESTS["test_m3_comprehensive.c"]="Testing globals and strings...
Global x = 42
String: Hello
Testing sizeof...
sizeof(int) = 2
sizeof(arr) = 10
sizeof(char) = 1
sizeof(ptr) = 2
Testing arrays and pointers...
arr[0] = 1
arr[1] = 2
arr[2] = 3
*ptr = 1
*(ptr+1) = 2
Modified arr[0] = 10
Done!"

# Additional tests that should at least compile without error
COMPILE_ONLY=(
    "test_struct_basic.c"
    "test_while_simple.c"
    "test_sizeof.c"
    "test_sizeof_verify.c"
    "test_pointers_comprehensive.c"
    "test_hello.c"
    "test_simple_putchar.c"
)

echo "Running tests with expected output..."
echo ""

# Function to run a single test
run_test() {
    local test_file=$1
    local expected_output=$2
    local test_name=$(basename "$test_file" .c)
    
    TOTAL=$((TOTAL + 1))
    
    printf "Testing %-30s ... " "$test_name"
    
    # Compile the test
    if ! $RCC compile "$TEST_DIR/$test_file" -o "$TEST_DIR/$test_name.asm" 2>/dev/null; then
        echo -e "${RED}FAILED${NC} (compilation error)"
        FAILED=$((FAILED + 1))
        return
    fi
    
    # Run the test if we have expected output
    if [ -n "$expected_output" ]; then
        # Run with rbt and capture output
        actual_output=$($RBT "$TEST_DIR/$test_name.asm" --run 2>/dev/null || true)
        
        # Compare output
        if [ "$actual_output" = "$expected_output" ]; then
            echo -e "${GREEN}PASSED${NC}"
            PASSED=$((PASSED + 1))
        else
            echo -e "${RED}FAILED${NC}"
            echo "  Expected: $(echo "$expected_output" | head -1)..."
            echo "  Got:      $(echo "$actual_output" | head -1)..."
            FAILED=$((FAILED + 1))
        fi
    else
        # Just check compilation
        echo -e "${GREEN}COMPILED${NC}"
        PASSED=$((PASSED + 1))
    fi
}

# Run tests with expected output
for test_file in "${!TESTS[@]}"; do
    if [ -f "$TEST_DIR/$test_file" ]; then
        run_test "$test_file" "${TESTS[$test_file]}"
    else
        printf "Testing %-30s ... " "$(basename "$test_file" .c)"
        echo -e "${YELLOW}SKIPPED${NC} (file not found)"
        SKIPPED=$((SKIPPED + 1))
        TOTAL=$((TOTAL + 1))
    fi
done

echo ""
echo "Running compile-only tests..."
echo ""

# Run compile-only tests
for test_file in "${COMPILE_ONLY[@]}"; do
    if [ -f "$TEST_DIR/$test_file" ]; then
        run_test "$test_file" ""
    else
        printf "Testing %-30s ... " "$(basename "$test_file" .c)"
        echo -e "${YELLOW}SKIPPED${NC} (file not found)"
        SKIPPED=$((SKIPPED + 1))
        TOTAL=$((TOTAL + 1))
    fi
done

echo ""
echo "=========================================="
echo "              Test Results                "
echo "=========================================="
echo ""
echo "Total:   $TOTAL"
echo -e "Passed:  ${GREEN}$PASSED${NC}"
echo -e "Failed:  ${RED}$FAILED${NC}"
echo -e "Skipped: ${YELLOW}$SKIPPED${NC}"
echo ""

if [ $FAILED -eq 0 ] && [ $SKIPPED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
elif [ $FAILED -eq 0 ]; then
    echo -e "${YELLOW}All run tests passed, but some were skipped.${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi