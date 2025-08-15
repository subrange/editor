// Test that nested struct types are properly resolved
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
    struct Middle* mid_ptr;
};

int main() {
    struct Outer o;
    struct Inner i;
    struct Middle m;
    
    i.x = 10;
    i.y = 20;
    
    m.inner.x = 30;
    m.inner.y = 40;
    m.ptr = &i;
    
    o.mid = m;
    o.mid_ptr = &m;
    
    // Test 1: Direct nested access
    if (o.mid.inner.x == 30) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Test 2: Pointer field access  
    if (o.mid.ptr->y == 20) {
        putchar('2');
    } else {
        putchar('N');
    }
    
    // Test 3: Through pointer to struct
    if (o.mid_ptr->inner.y == 40) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    // Test 4: Double pointer indirection
    if (o.mid_ptr->ptr->x == 10) {
        putchar('4');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}