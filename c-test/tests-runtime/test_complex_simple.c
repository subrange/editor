void putchar(int c);

int main() {
    int i = 0;
    int val = 10;
    
    // Test: (i == 0 && val == 10) should be (1 && 1) = 1
    if (i == 0 && val == 10) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Test: (i == 1 && val == 20) should be (0 && 0) = 0
    if (i == 1 && val == 20) {
        putchar('N');
    } else {
        putchar('2');
    }
    
    // Test: (i == 0 && val == 10) || (i == 1 && val == 20)
    // Should be 1 || 0 = 1
    if ((i == 0 && val == 10) || (i == 1 && val == 20)) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}