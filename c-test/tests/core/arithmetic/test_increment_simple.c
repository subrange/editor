// Simple test for increment operators
void putchar(int c);

int main() {
    int x = 5;
    
    // Test pre-increment
    ++x;
    if (x == 6) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Test post-increment
    x = 5;
    x++;
    if (x == 6) {
        putchar('2');
    } else {
        putchar('N');
    }
    
    // Test pre-decrement
    x = 5;
    --x;
    if (x == 4) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    // Test post-decrement
    x = 5;
    x--;
    if (x == 4) {
        putchar('4');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}