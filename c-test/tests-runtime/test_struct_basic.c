// Test inline struct - most basic
void putchar(int c);

int main() {
    struct {
        int x;
        int y;
    } p;
    
    p.x = 10;
    p.y = 20;
    
    int sum = p.x + p.y;
    
    if (sum == 30) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return sum;
}