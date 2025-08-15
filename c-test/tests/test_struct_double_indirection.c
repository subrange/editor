// Test for double indirection through struct pointers
// This test is extracted from test_struct_evil.c (Test 9)
void putchar(int c);

struct Inner {
    int x;
    int y;
};

struct Outer {
    struct Inner* nested_ptr;
};

int main() {
    struct Inner inner_obj;
    struct Outer outer;
    struct Outer* outer_ptr;
    
    // Initialize inner object
    inner_obj.x = 1000;
    inner_obj.y = 2000;
    
    // Set up the pointers
    outer.nested_ptr = &inner_obj;
    outer_ptr = &outer;
    
    // Test double indirection: outer_ptr->nested_ptr->y
    if (outer_ptr->nested_ptr->y == 2000) {
        putchar('Y');  // Success
    } else {
        putchar('N');  // Failure
    }
    
    putchar('\n');
    return 0;
}