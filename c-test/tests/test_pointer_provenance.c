

// Test pointer provenance with fat pointers
// This test ensures that pointers correctly track their memory bank (stack vs global)

void putchar(int c);

// Global variable
int global_var = 42;

// Function that takes a pointer parameter
// With fat pointers, this should work for both stack and global pointers
void print_value(int *ptr) {
    int val = *ptr;
    if (val == 42) {
        putchar('G');  // Global value
    } else if (val == 99) {
        putchar('S');  // Stack value
    } else {
        putchar('?');  // Unexpected
    }
}

int main() {
    // Test 1: Pass global pointer to function
    print_value(&global_var);  // Should print 'G'
    
    // Test 2: Pass stack pointer to function
    int stack_var = 99;
    print_value(&stack_var);   // Should print 'S'
    
    // Test 3: Array on stack passed to function
    int stack_array[3];
    stack_array[0] = 99;
    print_value(&stack_array[0]); // Should print 'S'
    
    // Test 4: Pointer arithmetic preserves bank
    int *ptr = &stack_var;
    int *ptr2 = ptr;  // Copy should preserve bank
    *ptr2 = 99;
    print_value(ptr2); // Should print 'S'
    
    putchar('\n');
    return 0;
}