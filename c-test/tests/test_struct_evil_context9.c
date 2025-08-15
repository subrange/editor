// Test 9 from test_struct_evil with more context
// This test fails in the full test but passes in isolation
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
    int data = 42;
    
    // Initialize local struct (same as original)
    local.a = 10;
    local.nested.x = 20;
    local.nested.y = 30;
    local.ptr = &data;
    local.arr[0] = 100;
    local.arr[1] = 200;
    local.arr[2] = 300;
    inner_obj.x = 1000;
    inner_obj.y = 2000;
    local.nested_ptr = &inner_obj;
    local.b = 40;
    
    // Test 7: Pointer to struct with complex access (modifies local.nested.x)
    evil_ptr = &local;
    evil_ptr->nested.x = 99;
    
    // Test 8: Array access through pointer (modifies local.arr[1])
    evil_ptr->arr[1] = 777;
    
    // Test 9: Double indirection
    if (evil_ptr->nested_ptr->y == 2000) {
        putchar('9');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}