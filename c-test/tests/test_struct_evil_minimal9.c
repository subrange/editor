// Minimal test for double indirection issue (Test 9 from test_struct_evil.c)
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
    
    // Initialize inner object
    inner_obj.x = 1000;
    inner_obj.y = 2000;
    
    // Set up the struct
    local.nested_ptr = &inner_obj;
    
    // Set up pointer to struct
    evil_ptr = &local;
    
    // Test 9: Double indirection - evil_ptr->nested_ptr->y
    if (evil_ptr->nested_ptr->y == 2000) {
        putchar('Y');  // Should print this
    } else {
        putchar('N');  // Currently prints this (failure)
    }
    
    putchar('\n');
    return 0;
}