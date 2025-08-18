// Test struct assignment with pointer field only
void putchar(int c);

struct Inner {
    int y;
};

struct Simple {
    struct Inner* ptr;   // Just a pointer field
};

int main() {
    struct Inner i;
    struct Simple s1, s2;
    
    i.y = 42;
    s1.ptr = &i;
    
    // This assignment should copy the fat pointer correctly
    s2 = s1;
    
    // Check if bank was preserved
    if (s2.ptr->y == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}