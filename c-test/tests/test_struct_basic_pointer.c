// Basic struct pointer test
void putchar(int c);

struct Simple {
    int x;
    int y;
};

int main() {
    struct Simple s;
    struct Simple* ptr;
    
    // Basic initialization
    s.x = 10;
    s.y = 20;
    
    // Get pointer to struct
    ptr = &s;
    
    // Test through pointer
    if (ptr->x == 10) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    if (ptr->y == 20) {
        putchar('2');
    } else {
        putchar('N');
    }
    
    // Modify through pointer
    ptr->x = 30;
    ptr->y = 40;
    
    // Test direct access
    if (s.x == 30) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    if (s.y == 40) {
        putchar('4');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}