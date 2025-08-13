void putchar(int c);

int main() {
    int a = 1;
    
    // Test (a == 1) && 1
    if ((a == 1) && 1) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}