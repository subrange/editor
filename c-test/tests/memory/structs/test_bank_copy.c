// Simplified test to debug bank copying
void putchar(int c);

struct Inner {
    int y;
};

struct Middle {
    struct Inner* ptr;   // Fat pointer: 2 words
};

int main() {
    struct Inner i;
    struct Middle m1, m2;
    
    i.y = 42;
    m1.ptr = &i;
    
    // This assignment should copy the fat pointer correctly
    m2 = m1;
    
    // Check if bank was preserved
    if (m2.ptr->y == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}