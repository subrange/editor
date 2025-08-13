void putchar(int c);

int main() {
    int a = 1;
    
    // Test variable == constant
    if (a == 1) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}