// Test direct access to pointer field with offset
void putchar(int c);

struct Inner {
    int x;
    int y;
};

struct Outer {
    int a;  // Field at offset 0
    struct Inner* ptr;  // Field at offset 1
};

int main() {
    struct Inner inner;
    struct Outer outer;
    
    inner.x = 10;
    inner.y = 20;
    
    outer.a = 5;
    outer.ptr = &inner;
    
    // Direct access through struct
    if (outer.ptr->y == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}