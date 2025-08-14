// Simple nested struct test - no chained member access
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
    
    // Get pointer to nested struct and initialize through pointer
    inner_ptr = &obj.inner;
    inner_ptr->a = 20;
    inner_ptr->b = 25;
    
    // Test outer fields
    if (obj.x == 10) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    if (obj.y == 30) {
        putchar('2');
    } else {
        putchar('N');
    }
    
    // Test nested fields through pointer
    if (inner_ptr->a == 20) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    if (inner_ptr->b == 25) {
        putchar('4');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}