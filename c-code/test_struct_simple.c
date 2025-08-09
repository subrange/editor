// Test basic struct support
void putchar(int c);

struct Point {
    int x;
    int y;
};

int main() {
    struct Point p;
    p.x = 10;
    p.y = 20;
    
    // Test p.x == 10
    if (p.x == 10) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Test p.y == 20
    if (p.y == 20) {
        putchar('2');
    } else {
        putchar('N');
    }
    
    // Test struct field assignment and access
    p.x = 30;
    if (p.x == 30) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    p.y = 40;
    if (p.y == 40) {
        putchar('4');
    } else {
        putchar('N');
    }
    
    // Test that fields are independent
    p.x = 50;
    if (p.x == 50 && p.y == 40) {
        putchar('5');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    
    return 0;
}