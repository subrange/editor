void putchar(int c);

int main() {
    int a = 1;
    int b = 0;
    
    // Simple OR test - should output '1'
    if (a || b) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Both false - should output '2'
    a = 0;
    b = 0;
    if (a || b) {
        putchar('N');
    } else {
        putchar('2');
    }
    
    // Both true - should output '3'
    a = 1;
    b = 1;
    if (a || b) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}
