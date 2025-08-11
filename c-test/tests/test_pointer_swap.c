void putchar(int c);

int main() {
    int a = 1;
    int b = 2;
    int *pa = &a;
    int *pb = &b;
    
    // Swap using pointers
    int temp = *pa;
    *pa = *pb;
    *pb = temp;
    
    // Check results
    if (a == 2) {
        putchar('A');
    } else {
        putchar('X');
    }
    
    if (b == 1) {
        putchar('B');
    } else {
        putchar('Y');
    }
    
    putchar('\n');
    return 0;
}