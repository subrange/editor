// Minimal test to reproduce test_struct_evil.c issue
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
    int data = 42;
    
    // Initialize local struct
    local.a = 10;
    local.ptr = &data;
    
    // Test 3: Pointer member dereference
    if (*local.ptr == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}