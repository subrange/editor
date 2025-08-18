void putchar(int c);

struct Inner {
    int x;
    int y; 
};

struct Outer {
    int a;
    struct Inner* ptr;
};

void print_num(int n) {
    if (n < 0) {
        putchar('-');
        n = -n;
    }
    if (n >= 10) {
        print_num(n / 10);
    }
    putchar('0' + (n % 10));
}

int main() {
    struct Inner inner;
    struct Outer outer;
    struct Outer* outer_ptr;
    
    inner.x = 10;
    inner.y = 20;
    
    outer.a = 5;
    outer.ptr = &inner;
    
    outer_ptr = &outer;
    
    // Debug: print values step by step
    putchar('a');
    putchar('=');
    print_num(outer.a);
    putchar(' ');
    
    putchar('x');
    putchar('=');
    print_num(outer.ptr->x);
    putchar(' ');
    
    putchar('y');
    putchar('=');
    print_num(outer.ptr->y);
    putchar(' ');
    
    // Now through pointer
    putchar('p');
    putchar('a');
    putchar('=');
    print_num(outer_ptr->a);
    putchar(' ');
    
    // The failing case
    putchar('p');
    putchar('y');
    putchar('=');
    int val = outer_ptr->ptr->y;
    print_num(val);
    putchar('\n');
    
    return 0;
}