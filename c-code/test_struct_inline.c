// Test inline struct
int main() {
    struct {
        int x;
        int y;
    } p;
    
    p.x = 10;
    p.y = 20;
    
    putchar('X');
    putchar(':');
    putchar('0' + p.x / 10);
    putchar('0' + p.x % 10);
    putchar(' ');
    
    putchar('Y');
    putchar(':');
    putchar('0' + p.y / 10);
    putchar('0' + p.y % 10);
    putchar('\n');
    
    return 0;
}