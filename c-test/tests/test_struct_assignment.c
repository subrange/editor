// Test struct assignment operation
void putchar(int c);

struct Point {
    int x;
    int y;
};

int main() {
    struct Point p1;
    p1.x = 10;
    p1.y = 20;
    
    // Test struct assignment
    struct Point p2;
    p2 = p1;  // This likely causes the timeout
    
    if (p2.x == 10 && p2.y == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return 0;
}