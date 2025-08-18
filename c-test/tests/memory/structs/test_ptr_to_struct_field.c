// Test accessing pointer field through pointer to struct  
void putchar(int c);

struct Inner {
    int x;
    int y;
};

struct Outer {
    int a;
    struct Inner* ptr;
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
    
    // Test through pointer to struct
    if (outer_ptr->ptr->y == 20) {
        putchar('Y');
    } else {
        putchar('N');  
    }
    
    putchar('\n');
    return 0;
}