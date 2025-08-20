// Test specifically for o.mid.ptr->y pattern
void putchar(int c);

struct Inner {
    int x;
    int y;
};

struct Middle {
    struct Inner inner;  // Direct struct field
    struct Inner* ptr;   // Pointer to struct field
};

struct Outer {
    struct Middle mid;
};

int main() {
    struct Inner i;
    struct Middle m;
    struct Outer o;
    
    i.x = 10;
    i.y = 20;
    
    m.inner.x = 30;
    m.inner.y = 40;
    m.ptr = &i;
    
    o.mid = m;

    // This works
    if (o.mid.inner.y == 40) {
        putchar('Y');
    } else {
        putchar('N');
    }

    // This is the failing test case
    if (o.mid.ptr->y == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}