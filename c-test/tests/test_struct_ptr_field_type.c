// Test that struct pointer fields maintain correct type
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
    struct Outer* outer_ptr;
    
    inner.x = 10;
    inner.y = 20;
    
    outer.a = 5;
    outer.ptr = &inner;
    
    outer_ptr = &outer;
    
    // Direct access through struct
    if (outer.ptr->y == 20) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Access through pointer
    if (outer_ptr->ptr->y == 20) {
        putchar('2');
    } else {
        putchar('N');  
    }
    
    putchar('\n');
    return 0;
}