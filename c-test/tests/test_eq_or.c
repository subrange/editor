void putchar(int c);

int main() {
    int a = 1;
    int b = 0;
    
    // Test: (a == 1) || (b == 1) - should output '1' (true || false = true)
    if ((a == 1) || (b == 1)) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Test: (a == 0) || (b == 0) - should output '2' (false || true = true)  
    if ((a == 0) || (b == 0)) {
        putchar('2');
    } else {
        putchar('N');
    }
    
    // Test: (a == 0) || (b == 1) - should output 'N' (false || false = false)
    if ((a == 0) || (b == 1)) {
        putchar('N');
    } else {
        putchar('3');
    }
    
    putchar('\n');
    return 0;
}