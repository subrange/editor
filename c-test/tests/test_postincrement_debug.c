// Debug post-increment
void putchar(int c);

int main() {
    int x = 5;
    int y;
    
    // Test post-increment
    y = x++;  // y should get 5, x should become 6
    
    // Print the values
    putchar('0' + y);  // Should print '5'
    putchar(',');
    putchar('0' + x);  // Should print '6'
    
    putchar('\n');
    return 0;
}