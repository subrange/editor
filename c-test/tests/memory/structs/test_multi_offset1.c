// Test pointer to multi-field struct at offset 1
void putchar(int c);

struct Inner {
    int x;
    int y;
};

struct Container {
    int dummy;  // Force ptr to be at offset 1
    struct Inner* ptr;
};

int main() {
    struct Inner inner;
    struct Container cont;
    
    inner.x = 10;
    inner.y = 20;
    cont.dummy = 99;
    cont.ptr = &inner;
    
    // Check x
    if (cont.ptr->x == 10) {
        putchar('X');
    } else {
        putchar('x');
    }
    
    // Check y
    if (cont.ptr->y == 20) {
        putchar('Y');
    } else {
        putchar('y');
    }
    
    putchar('\n');
    return 0;
}