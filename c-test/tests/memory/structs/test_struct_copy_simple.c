void putchar(int c);

struct Point {
    int x;
    int y;
};

int main() {
    struct Point a;
    struct Point b;
    
    a.x = 10;
    a.y = 20;
    
    b = a;  // Simple struct assignment
    
    if (b.x == 10) {
        putchar('1');
    } else {
        putchar('0');
    }
    
    if (b.y == 20) {
        putchar('2');
    } else {
        putchar('0');
    }
    
    putchar('\n');
    return 0;
}