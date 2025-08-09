// Test typedef support - currently not fully implemented
// The parser needs to track typedef names during parsing to distinguish
// between typedef'd types and regular identifiers (the classic C parsing problem)

typedef int myint;
typedef struct {
    int x;
    int y;
} Point;

int main() {
    myint a = 10;
    Point p;
    p.x = 20;
    p.y = 30;
    
    // Should print: 10 20 30
    putchar('0' + a / 10);
    putchar('0' + a % 10);
    putchar(' ');
    putchar('0' + p.x / 10);
    putchar('0' + p.x % 10);
    putchar(' ');
    putchar('0' + p.y / 10);
    putchar('0' + p.y % 10);
    putchar('\n');
    
    return 0;
}