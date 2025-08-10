// Test inline struct
void putchar(int c);

int main() {
    struct {
        int x;
        int y;
    } p;
    
    p.x = 10;
    p.y = 20;
    
    // Test struct member x
    if (p.x == 10) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test struct member y
    if (p.y == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    
    return 0;
}