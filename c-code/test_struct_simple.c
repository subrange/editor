// Test basic struct support
struct Point {
    int x;
    int y;
};

int main() {
    struct Point p;
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
    
    // Test pointer to struct
    struct Point *ptr = &p;
    ptr->x = 30;
    ptr->y = 40;
    
    putchar('P');
    putchar(':');
    putchar('0' + ptr->x / 10);
    putchar('0' + ptr->x % 10);
    putchar(',');
    putchar('0' + ptr->y / 10);
    putchar('0' + ptr->y % 10);
    putchar('\n');
    
    return 0;
}