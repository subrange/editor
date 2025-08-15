// Test struct pointer member assignment
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

struct Evil global_evil;

int main() {
    struct Evil local;
    struct Inner inner_obj;
    
    // Initialize
    inner_obj.x = 1000;
    inner_obj.y = 2000;
    local.nested_ptr = &inner_obj;
    
    // Test 5: Pointer to nested struct
    if (local.nested_ptr->x == 1000) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}