// Test struct arrays
void putchar(int c);
void puts(char *s);

struct Point {
    int x;
    int y;
};

int main() {
    puts("Test struct array");
    
    struct Point points[3];
    points[0].x = 10;
    points[0].y = 20;
    points[1].x = 30;
    points[1].y = 40;
    points[2].x = 50;
    points[2].y = 60;
    
    if (points[1].x == 30) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return 0;
}