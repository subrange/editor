// Test taking address of nested struct field
void putchar(int c);

struct Inner {
    int a;
    int b;
};

struct Outer {
    int x;
    struct Inner inner;
    int y;
};

int main() {
    struct Outer obj;
    struct Inner* inner_ptr;
    
    // Initialize outer fields  
    obj.x = 10;
    obj.y = 30;
    
    // Initialize inner fields directly first
    obj.inner.a = 20;
    obj.inner.b = 25;
    
    // Test direct access to nested fields
    if (obj.inner.a == 20) {
        putchar('1');  // Should print
    } else {
        putchar('N');
    }
    
    if (obj.inner.b == 25) {
        putchar('2');  // Should print
    } else {
        putchar('N');
    }
    
    // Get pointer to nested struct
    inner_ptr = &obj.inner;
    
    // Test access through pointer
    if (inner_ptr->a == 20) {
        putchar('3');  // Should print
    } else {
        putchar('N');
    }
    
    if (inner_ptr->b == 25) {
        putchar('4');  // Should print
    } else {
        putchar('N');
    }
    
    // Test modification through pointer
    inner_ptr->a = 100;
    inner_ptr->b = 200;
    
    if (obj.inner.a == 100) {
        putchar('5');  // Should print
    } else {
        putchar('N');
    }
    
    if (obj.inner.b == 200) {
        putchar('6');  // Should print
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}