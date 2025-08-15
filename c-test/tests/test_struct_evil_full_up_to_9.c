// Full test up to and including test 9
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

// Global struct for even more evil
struct Evil global_evil;

int main() {
    struct Evil local;
    struct Evil* evil_ptr;
    int data = 42;
    int i;
    struct Inner inner_obj;
    
    // Initialize local struct
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
    
    // Test 1: Basic field access
    if (local.a == 10 && local.b == 40) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Test 2: Nested struct access
    if (local.nested.x == 20 && local.nested.y == 30) {
        putchar('2');
    } else {
        putchar('N');
    }
    
    // Test 3: Pointer member dereference
    if (*local.ptr == 42) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    // Test 4: Array in struct
    if (local.arr[0] + local.arr[1] + local.arr[2] == 600) {
        putchar('4');
    } else {
        putchar('N');
    }
    
    // Test 5: Pointer to nested struct
    if (local.nested_ptr->x == 1000) {
        putchar('5');
    } else {
        putchar('N');
    }
    
    // Test 6: Global struct access
    global_evil.a = 77;
    global_evil.nested.x = 88;
    if (global_evil.a == 77 && global_evil.nested.x == 88) {
        putchar('6');
    } else {
        putchar('N');
    }
    
    // Test 7: Pointer to struct with complex access
    evil_ptr = &local;
    evil_ptr->nested.x = 99;
    if (local.nested.x == 99) {
        putchar('7');
    } else {
        putchar('N');
    }
    
    // Test 8: Array access through pointer
    evil_ptr->arr[1] = 777;
    if (local.arr[1] == 777) {
        putchar('8');
    } else {
        putchar('N');
    }
    
    // Test 9: Double indirection
    if (evil_ptr->nested_ptr->y == 2000) {
        putchar('9');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}