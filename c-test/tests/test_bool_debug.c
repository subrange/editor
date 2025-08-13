void putchar(int c);

int main() {
    int a = 1;
    int b = 2;  // Different value to help debug
    
    // First test a
    if (a == 1) {
        putchar('A');
    } else {
        putchar('X');
    }
    
    // Then test b  
    if (b == 2) {
        putchar('B');
    } else {
        putchar('Y');
    }
    
    // Now test AND
    if (a == 1 && b == 2) {
        putchar('C');
    } else {
        putchar('Z');
    }
    
    putchar('\n');
}