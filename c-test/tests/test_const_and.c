void putchar(int c);

int main() {
    // Test constant AND
    if (1 && 1) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}