void putchar(int c);

int main() {
    int a = 1;
    int b = 1;
    
    // Test (a == 1) && (b == 1) without any function calls before
    if ((a == 1) && (b == 1)) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}