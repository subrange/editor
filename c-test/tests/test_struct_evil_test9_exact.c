// Exact reproduction of test 9 failure from test_struct_evil.c
// Tests that match the exact pattern that fails
void putchar(int c);

struct Inner {
    int x;
    int y;
};

struct Evil {
    int a;
    struct Inner nested;
    int* ptr;
    int arr[3];
    struct Inner* nested_ptr;
    int b;
};

int main() {
    struct Evil local;
    struct Evil* evil_ptr;
    struct Inner inner_obj;
    
    // Initialize exactly as in the original test
    inner_obj.x = 1000;
    inner_obj.y = 2000;
    local.nested_ptr = &inner_obj;
    
    // Test 7 operation (modifies nested.x through pointer)
    evil_ptr = &local;
    evil_ptr->nested.x = 99;
    
    // Test 8 operation (modifies arr[1] through pointer)
    evil_ptr->arr[1] = 777;
    
    // Now test 9: Double indirection with both operations preceding it
    // This is what's failing in the full test
    if (evil_ptr->nested_ptr->y == 2000) {
        putchar('Y');  // Should print this
    } else {
        putchar('N');  // Failing test prints this
    }
    
    // Also test if direct access works
    if (local.nested_ptr->y == 2000) {
        putchar('D');  // Direct access
    } else {
        putchar('X');
    }
    
    putchar('\n');
    return 0;
}