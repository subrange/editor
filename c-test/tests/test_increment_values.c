// Test increment and decrement operators with value checking
void putchar(int c);

int main() {
    int x = 5;
    int y;
    
    // Test pre-increment
    y = ++x;  // x becomes 6, y becomes 6
    if (x == 6 && y == 6) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test post-increment
    x = 5;
    y = x++;  // y becomes 5, x becomes 6
    if (x == 6 && y == 5) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test pre-decrement
    x = 5;
    y = --x;  // x becomes 4, y becomes 4
    if (x == 4 && y == 4) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test post-decrement
    x = 5;
    y = x--;  // y becomes 5, x becomes 4
    if (x == 4 && y == 5) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}