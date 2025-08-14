// Test nested struct support
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
    
    // Initialize nested structure
    obj.x = 10;
    obj.inner.a = 20;
    obj.inner.b = 30;
    obj.y = 40;
    
    // Test outer field before nested
    if (obj.x == 10) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Test nested fields
    if (obj.inner.a == 20) {
        putchar('2');
    } else {
        putchar('N');
    }
    
    if (obj.inner.b == 30) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    // Test outer field after nested
    if (obj.y == 40) {
        putchar('4');
    } else {
        putchar('N');
    }
    
    // Modify nested struct and verify
    obj.inner.a = 50;
    obj.inner.b = 60;
    
    if (obj.inner.a == 50 && obj.inner.b == 60) {
        putchar('5');
    } else {
        putchar('N');
    }
    
    // Verify outer fields unchanged
    if (obj.x == 10 && obj.y == 40) {
        putchar('6');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}