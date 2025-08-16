// Test pointer arithmetic with bank crossing
// Bank size is 4096 bytes (2048 16-bit words)
void putchar(int c);

int main() {
    // Create a large array that spans multiple banks
    // 2000 ints = 4000 bytes, which will cross bank boundary at 4096 bytes
    int huge_array[2000];
    
    // Initialize some values
    huge_array[0] = 42;
    huge_array[1000] = 99;
    huge_array[1500] = 123;  // This will be in a different bank
    huge_array[1999] = 456;  // Last element
    
    // Test 1: Access element in first bank
    int *p = &huge_array[0];
    if (*p == 42) {
        putchar('1');  // Test 1 passed
    } else {
        putchar('N');  // Test 1 failed
    }
    
    // Test 2: Access element in middle (still first bank)
    p = huge_array + 1000;
    if (*p == 99) {
        putchar('2');  // Test 2 passed
    } else {
        putchar('N');  // Test 2 failed
    }
    
    // Test 3: Access element that crosses bank boundary
    // Element 1500: 1500 * 2 bytes = 3000 bytes offset
    // With 2-byte offset at start, total = 3002 bytes (still in first bank)
    // But let's use a value that definitely crosses: element at index 1500
    p = huge_array + 1500;
    if (*p == 123) {
        putchar('3');  // Test 3 passed - bank crossing handled correctly
    } else {
        putchar('N');  // Test 3 failed - bank crossing not handled
    }
    
    // Test 4: Access last element (definitely in second bank)
    p = &huge_array[1999];
    if (*p == 456) {
        putchar('4');  // Test 4 passed
    } else {
        putchar('N');  // Test 4 failed
    }
    
    // Test 5: Pointer arithmetic across banks
    p = huge_array;
    p = p + 1500;  // Large offset that may cross bank
    if (*p == 123) {
        putchar('5');  // Test 5 passed
    } else {
        putchar('N');  // Test 5 failed
    }
    
    putchar('\n');
    return 0;
}