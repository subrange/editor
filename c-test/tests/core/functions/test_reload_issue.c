void putchar(int c);

int main() {
    int a = 1;
    
    // First check - this works
    if (a == 1) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Second check after function call - does this work?
    if (a == 1) {
        putchar('2');
    } else {
        putchar('M');
    }
    
    putchar('\n');
    return 0;
}