// Debug version of test_struct_ptr_field_type
void putchar(int c);

struct Inner {
    int x;
    int y; 
};

struct Outer {
    int a;
    struct Inner* ptr;  // This should be a pointer, not array
};

int main() {
    struct Inner inner;
    struct Outer outer;
    
    inner.x = 10;
    inner.y = 20;
    
    outer.a = 5;
    outer.ptr = &inner;
    
    // Debug: Check if ptr is non-null
    if (outer.ptr) {
        putchar('1');  // Ptr is set
    } else {
        putchar('0');  // Ptr is null
    }
    
    // Debug: Try to access x first
    if (outer.ptr->x == 10) {
        putchar('X');
    } else {
        putchar('x');
    }
    
    // Original test: Direct access through struct  
    if (outer.ptr->y == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}