// Minimal nested struct test
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
    
    // Initialize
    obj.x = 10;
    obj.inner.a = 20;  // This is what's failing
    obj.inner.b = 30;
    obj.y = 40;
    
    // Test nested field a
    if (obj.inner.a == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}