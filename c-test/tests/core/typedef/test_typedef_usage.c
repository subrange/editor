void putchar(int c);

typedef int myint;
typedef char* string;
typedef struct Point {
    int x;
    int y;
} Point;

int main() {
    // Test basic typedef usage
    myint a = 42;
    myint b = 58;
    myint sum = a + b;
    
    if (sum == 100) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test pointer typedef
    string message = "Hello";
    putchar(message[0]); // 'H'
    
    // Test struct typedef  
    Point p;
    p.x = 10;
    p.y = 20;
    
    if (p.x + p.y == 30) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}